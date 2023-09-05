use std::ops::{Deref, DerefMut};

use asefile::{ColorPalette, ColorPaletteEntry};
use id_tree::InsertBehavior::*;
use id_tree::*;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

use super::binpack::{binpack, Bin, BinItem};

static MAX_PAL_SIZE: usize = 256;
pub static PAL_BANK_SIZE: usize = 16;

#[derive(Clone, Debug)]
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

impl From<&ColorPalette> for Palette {
    fn from(color_palette: &ColorPalette) -> Self {
        let num_colors = color_palette.num_colors();

        let palette = (0..num_colors).map(|i| {
            let pal_entry = color_palette.color(i).unwrap();
            palette_entry_to_15bit_color(pal_entry)
        });

        let palette: Vec<u16> = palette.collect();

        Self { _palette: palette }
    }
}

impl ToTokens for Palette {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let tiles: Vec<String> = self
            .iter()
            .map(|tile| quote! { gba::video::Color(#tile) }.to_string())
            .collect();

        let tiles = tiles.join(",");
        let tiles = format!("[ {} ]", tiles);

        let expr: syn::Expr =
            syn::parse_str(&tiles).expect("Error producing hex representation of palette colors.");
        expr.to_tokens(tokens);
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

pub fn resolve_palette(palette_tree: Tree<Palette>) -> PaletteMapper {
    // First sort the top layer of palettes by size.
    let root_id = palette_tree.root_node_id();

    let root_id = match root_id {
        Some(id) => id,
        None => panic!("Tree not initialized"),
    }
    .clone();

    // Get all of the root palettes, which encompass every palette in the game.
    let root_node = palette_tree.get(&root_id).unwrap();
    let root_palettes: Vec<Palette> = root_node
        .children()
        .iter()
        .map(|node_id| palette_tree.get(node_id).unwrap().data())
        .cloned()
        .collect();

    // Pack them into "bins", representing the palette banks.
    let root_palettes: Vec<BinItem<Palette>> = root_palettes
        .iter()
        .map(|p| BinItem::new(p.clone(), p.len()))
        .collect();

    let palette_banks = binpack(root_palettes, PAL_BANK_SIZE - 1).unwrap();

    let max_banks = MAX_PAL_SIZE / PAL_BANK_SIZE;
    assert!(palette_banks.len() <= max_banks);

    // Now unflatten and pad out the palettes so they are aligned to the palette banks.
    let palette_banks = flatten_palette_banks(palette_banks);

    PaletteMapper::from(palette_banks)
}

fn flatten_palette_banks(palette_banks: Vec<Bin<Palette>>) -> Vec<Palette> {
    let mut flattened_palettes: Vec<Palette> = Vec::new();

    for bank in palette_banks {
        let bank: Vec<Palette> = bank.into();
        let bank = bank.into_iter();

        let flatbank: Option<Palette> = bank.reduce(|mut s, mut n| {
            (s).append(&mut n);
            s
        });

        if let Some(mut bank) = flatbank {
            // Every bank needs to start with zero, because we can never
            // access the zeroth element from a 4BPP tile.
            bank.insert(0, 0);

            // // Pad out the bank with zeroes to reach the target size.
            bank.resize(PAL_BANK_SIZE, 0);
            flattened_palettes.push(bank);
        }
    }

    flattened_palettes
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

impl From<Vec<Palette>> for PaletteMapper {
    fn from(palettes: Vec<Palette>) -> Self {
        let mut final_palette: [u16; 256] = [0; 256];

        for (i, pal) in palettes.iter().enumerate() {
            let slice = &mut final_palette[i * PAL_BANK_SIZE..(i * PAL_BANK_SIZE + PAL_BANK_SIZE)];

            for (j, color) in pal.iter().enumerate() {
                slice[j] = *color;
            }
        }

        PaletteMapper { final_palette }
    }
}

pub fn palette_entry_to_15bit_color(palette_entry: &ColorPaletteEntry) -> u16 {
    // Convert each channel from 8-bit into 5-bit (shift right by 3)
    // Then wrap each channel in a u16 so we can combine them.
    let red: u16 = u16::from(palette_entry.red() >> 3);
    let green: u16 = u16::from(palette_entry.green() >> 3);
    let blue: u16 = u16::from(palette_entry.blue() >> 3);

    red | (green << 5) | (blue << 10)
}

impl PaletteMapper {
    pub fn map_palette(
        &self,
        raw_palette: &Palette,
        transparency_index: Option<u8>,
    ) -> Option<MappedPalette> {
        // Make sure the palette can fit into a palbank.
        if raw_palette.len() > MAX_PAL_SIZE {
            return None;
        }

        // Find the palette bank that contains this palette
        self.final_palette
            .chunks(PAL_BANK_SIZE)
            .enumerate()
            .filter(|(_i, d)| raw_palette.iter().all(|c| d.contains(c)))
            .map(|(i, d)| {
                let mut indices: [u8; 16] = [0; 16];

                for raw_color_index in 0..raw_palette.len() {
                    let color = raw_palette[raw_color_index];

                    // Find color in the final palbank
                    let new_index = d.iter().position(|c| *c == color).unwrap();

                    indices[raw_color_index] = new_index.try_into().unwrap();
                }

                MappedPalette {
                    pal_bank: i.try_into().unwrap(),
                    indices,
                    transparency_index,
                }
            })
            .next()
    }

    pub fn full_palette(&self) -> Palette {
        Palette::new(Vec::from(self.final_palette))
    }
}

impl MappedPalette {
    pub fn map_index(&self, index: u8) -> u8 {
        if let Some(n) = self.transparency_index {
            if index == n {
                return 0;
            }
        }
        let index: usize = index.into();

        self.indices[index]
    }

    pub fn palette_bank(&self) -> u8 {
        self.pal_bank
    }
}
