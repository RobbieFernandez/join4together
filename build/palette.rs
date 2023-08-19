use std::collections::VecDeque;
use std::ops::{Deref, DerefMut};

use id_tree::InsertBehavior::*;
use id_tree::*;

static MAX_PAL_SIZE: usize = 256;
pub static PAL_BANK_SIZE: usize = 16;

#[derive(Clone)]
pub struct Palette {
    _palette: Vec<u16>,
}

impl Palette {
    pub fn new(colors: Vec<u16>) -> Self {
        Self { _palette: colors }
    }

    fn contains(&self, other: &Self) -> bool {
        other.iter().all(|c| self._palette.contains(c))
    }
}

impl Deref for Palette {
    type Target = Vec<u16>;

    fn deref(&self) -> &Self::Target {
        &self._palette
    }
}

impl DerefMut for Palette {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self._palette
    }
}

#[derive(Debug)]
pub struct PaletteMapper {
    pub final_palette: [u16; 256],
}

pub struct MappedPalette {
    pal_bank: u8,
    pub indices: [u8; 16],
    transparency_index: Option<u8>,
}

pub fn add_palette(palette_tree: &mut Tree<Palette>, palette: Palette) {
    let root_id = palette_tree.root_node_id();

    let root_id = match root_id {
        Some(id) => id,
        None => panic!("Tree not initialized"),
    }
    .clone();

    let root_node = palette_tree.get(&root_id).unwrap();
    let top_palettes = root_node.children();

    let palette_subtree = top_palettes.iter().find(|id| {
        let root_palette = palette_tree.get(id).unwrap();
        root_palette.data().contains(&palette)
    });

    // Move this palette into the appropriate level
    let new_node_id = match palette_subtree {
        Some(subtree_id) => {
            let mut iter = palette_tree.traverse_post_order_ids(subtree_id).unwrap();

            let parent_palette_id = iter.find(|id| {
                let this_palette = palette_tree.get(id).unwrap();
                this_palette.data().contains(&palette)
            });

            let new_node = Node::new(palette);

            if let Some(id) = parent_palette_id {
                palette_tree.insert(new_node, UnderNode(&id)).unwrap()
            } else {
                let subtree_id = subtree_id.clone();

                palette_tree
                    .insert(new_node, UnderNode(&subtree_id))
                    .unwrap()
            }
        }
        None => {
            let new_node = Node::new(palette);
            palette_tree.insert(new_node, UnderNode(&root_id)).unwrap()
        }
    };

    // Siblings of this palette may need to be moved underneath it.
    let new_node = palette_tree.get(&new_node_id).unwrap();
    let parent = new_node.parent().unwrap();
    let siblings_to_move: Vec<NodeId> = palette_tree
        .children_ids(parent)
        .unwrap()
        .filter(|id| **id != new_node_id)
        .filter(|id| {
            let sibling = palette_tree.get(id).unwrap();
            new_node.data().contains(sibling.data())
        })
        .cloned()
        .collect();

    for sibling_id in siblings_to_move {
        palette_tree
            .move_node(&sibling_id, MoveBehavior::ToParent(&new_node_id))
            .unwrap();
    }
}

pub fn resolve_palette(mut palette_tree: Tree<Palette>) -> PaletteMapper {
    // First sort the top layer of palettes by size.
    let root_id = palette_tree.root_node_id();

    let root_id = match root_id {
        Some(id) => id,
        None => panic!("Tree not initialized"),
    }
    .clone();

    // Sort the top-level palette by length.
    merge_top_level_palettes(&mut palette_tree, &root_id);

    // Now pad out the palettes so they are aligned to palbanks.
    align_palette_banks(&mut palette_tree, &root_id);

    // Make sure the combined length of all palettes will fit inside PALRAM.
    let children: Vec<&NodeId> = palette_tree.children_ids(&root_id).unwrap().collect();
    let total_pal_size = children.len() * PAL_BANK_SIZE;

    if total_pal_size > MAX_PAL_SIZE {
        panic!("Palette could not be optimized to fit entirely in PALRAM");
    }

    PaletteMapper::from(palette_tree)
}

fn merge_top_level_palettes(palette_tree: &mut Tree<Palette>, root_id: &NodeId) {
    palette_tree
        .sort_children_by_key(root_id, |n| n.data().len())
        .unwrap();

    let root_node = palette_tree.get(root_id).unwrap();

    let top_palette_node_ids = root_node.children().clone();
    let mut top_palette_node_ids = VecDeque::from(top_palette_node_ids);

    if top_palette_node_ids.is_empty() {
        return;
    }

    // Combine adjacent palettes, so long as they can fit into a single palbank
    let mut current_node_id = top_palette_node_ids.pop_front().unwrap();
    let mut done = false;

    while !done {
        let next_palette = top_palette_node_ids.pop_front();

        match next_palette {
            Some(next_palette_id) => {
                let mut next_palette = palette_tree.get(&next_palette_id).unwrap().data().clone();

                let current_palette_length =
                    palette_tree.get(&current_node_id).unwrap().data().len();

                let combined_length = current_palette_length + next_palette.len();

                if combined_length < PAL_BANK_SIZE {
                    // Combine these palettes.
                    {
                        let mut current_palette =
                            palette_tree.get(&current_node_id).unwrap().data().clone();

                        current_palette.append(&mut next_palette);
                    }

                    // Move the children from the next palette into the current one.
                    let children_ids: Vec<NodeId> = palette_tree
                        .children_ids(&next_palette_id)
                        .unwrap()
                        .cloned()
                        .collect();

                    for child_id in children_ids {
                        palette_tree
                            .move_node(&child_id, MoveBehavior::ToParent(&current_node_id))
                            .unwrap();
                    }

                    palette_tree
                        .remove_node(next_palette_id, RemoveBehavior::DropChildren)
                        .unwrap();
                } else {
                    // Begin processing the next palette.
                    current_node_id = next_palette_id;
                }
            }
            None => {
                done = true;
            }
        }
    }
}

fn align_palette_banks(palette_tree: &mut Tree<Palette>, root_id: &NodeId) {
    let root_node = palette_tree.get(root_id).unwrap();
    let top_palette_node_ids = root_node.children().clone();

    for node_id in top_palette_node_ids {
        let palette = palette_tree.get_mut(&node_id).unwrap().data_mut();

        // Use ">=" because index 0 always means transparent, so the palette can actually
        // only hold 15 colours.
        if palette.len() >= PAL_BANK_SIZE {
            panic!("We have created a palbank that is too big!");
        }

        // A color index of zero always means transparent.
        // In 4BPP mode, this means we can never use the 0th element
        // of any palette bank. So always add a zero to the front
        // of the bank, making the actual colours start at element 1.
        palette.insert(0, 0);

        palette.resize(PAL_BANK_SIZE, 0);
    }
}

impl From<Tree<Palette>> for PaletteMapper {
    fn from(palette_tree: Tree<Palette>) -> Self {
        let mut final_palette: [u16; 256] = [0; 256];

        let root_id = palette_tree.root_node_id();

        let root_id = match root_id {
            Some(id) => id,
            None => panic!("Tree not initialized"),
        }
        .clone();

        let children = palette_tree.children(&root_id).unwrap();

        for (i, child) in children.enumerate() {
            let slice = &mut final_palette[i * PAL_BANK_SIZE..(i * PAL_BANK_SIZE + PAL_BANK_SIZE)];

            let pal_bank_colors = child.data();

            for (j, color) in pal_bank_colors.iter().enumerate() {
                slice[j] = *color;
            }
        }

        PaletteMapper { final_palette }
    }
}

impl PaletteMapper {
    pub fn map_palette(
        &self,
        raw_palette: &Palette,
        transparency_index: Option<u8>,
    ) -> Option<MappedPalette> {
        // Make sure the palette can fit into a palbank.
        if raw_palette.len() > MAX_PAL_SIZE {
            panic!("Palette cannot contain more than {} colors.", MAX_PAL_SIZE);
        }

        self.final_palette
            .chunks(MAX_PAL_SIZE)
            .enumerate()
            .filter(|(_i, d)| raw_palette.iter().all(|c| d.contains(c)))
            .map(|(i, d)| {
                let mut indices: [u8; 16] = [0; 16];

                for raw_color_index in 0..raw_palette.len() {
                    let color = raw_palette[raw_color_index];

                    // Find color in the final palbank
                    let new_index = d.iter().position(|c| *c == color).unwrap();

                    indices[raw_color_index] = new_index as u8;
                }

                MappedPalette {
                    pal_bank: i as u8,
                    indices,
                    transparency_index,
                }
            })
            .next()
    }

    pub fn full_palette(&self) -> Palette {
        Palette::new(
            Vec::from(self.final_palette.clone())
        )
    }
}

impl MappedPalette {
    pub fn map_index(&self, index: u8) -> u8 {
        if let Some(n) = self.transparency_index {
            if index == n {
                return 0;
            }
        }

        self.indices[index as usize]
    }

    pub fn palette_bank(&self) -> u8 {
        self.pal_bank
    }
}
