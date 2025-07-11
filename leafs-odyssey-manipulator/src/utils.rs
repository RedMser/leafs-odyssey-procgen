use leafs_odyssey_data::{builder::{TileSelection, Tilemap}, data::*};

pub fn write_tiles<F>(to_write: Vec<LOTile>, current_layers: &mut Vec<LOLayer>, selection_fn: F) -> Result<(), String>
where
    F: FnOnce(&Tilemap) -> TileSelection
{
    if to_write.len() > current_layers.len() {
        return Err(format!("More tiles than layers were specified: {} > {}", to_write.len(), current_layers.len()).to_string());
    }

    let mut tilemap = Tilemap::from(&mut *current_layers);
    let selection = selection_fn(&tilemap);

    if to_write.len() == current_layers.len() || to_write.len() >= 5 {
        // Full replace of this coordinate.
        for layer in 0..to_write.len() {
            tilemap.write_on_layer(layer, &to_write[layer], &selection);
        }
    } else {
        // Smart placement of tiles.
        for tile in to_write {
            tilemap.write(&tile, &selection);
        }
    }

    let modified = tilemap.into_layers();
    for (i, mod_layer) in modified.into_iter().enumerate() {
        current_layers[i].width = mod_layer.width;
        current_layers[i].height = mod_layer.height;
        current_layers[i].tiles = mod_layer.tiles;
    }

    Ok(())
}

/// Creates a selection that matches every tile in current_layers which fit the predicate tiles list.
pub fn conditional_tile_selection(tilemap: &Tilemap, predicate: &Vec<LOTile>) -> Result<TileSelection, String> {
    if predicate.len() > tilemap.layers.len() {
        return Err(format!("More condition tiles than layers were specified: {} > {}", predicate.len(), tilemap.layers.len()).to_string());
    }

    let selection = if predicate.len() == tilemap.layers.len() {
        // Full match of all layers.
        tilemap.select_if_all_layers_equal(predicate)
    } else {
        // AND-check on any layers.
        tilemap.select_if_all_exist(predicate)
    };

    Ok(selection)
}