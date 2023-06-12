use std::ops::Deref;
use std::sync::MutexGuard;
use std::time::Instant;

use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;

use bevy::sprite::MaterialMesh2dBundle;
use bevy::utils::HashMap;

use crate::data::{GameState, ServerGameState, Position, Size, PlayerId, VersionId, PreviousPos};

pub struct SnekViewerPlugin;

impl Plugin for SnekViewerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            (
                update_entities_from_server_state,
                //render_everything,
                position_translation,
                size_scaling
            ).chain()
        );
    }
}


fn convert(pos: f32, bound_window: f32, bound_game: f32) -> f32 {
    let tile_size = bound_window / bound_game;
    pos / bound_game * bound_window // - (bound_window / 2.) + (tile_size / 2.)
}

fn update_entities_from_server_state(
    mut commands: Commands,
    mut state_res: ResMut<ServerGameState>,
    windows: Query<&Window>,
    mut player_entities: Query<(Entity, &PlayerId, &mut Sprite)>,
) {
    let window = windows.get_single().unwrap();

    let mut state = {
        let lock = state_res.game_state.lock().unwrap();
        lock.clone()
    };

    if state_res.current_game_id != state.id {
        state_res.current_game_id = state.id;
        state.version = 1;
        info!("New game started!");

        // for (entity, _, _) in player_entities.iter() {
        //     commands.entity(entity).despawn();
        // }
    }

    // The server state has not changed since we last updated the entities,
    // so there's nothing to do
    if state.version == state_res.current_version {
        return;
    }

    state_res.current_version = state.version;
    let elapsed = state_res.last_update.elapsed();
    debug!("Applying ({}/{}) ({}ms since last update)", state_res.current_game_id, state_res.current_version, elapsed.as_millis());

    // for (entity, id, mut sprite) in player_entities.iter_mut() {
    //     let player = state.players.get(id.0).unwrap();
    //     if !player.alive {
    //         commands.entity(entity).despawn();
    //         continue;
    //     }
    //
    //     let segment_color = Color::hsl((id.0 as f32)  * 30.0, 1.0, 0.8);
    //     sprite.color = segment_color;
    // }

    for (entity, _, _) in player_entities.iter() {
        commands.entity(entity).despawn();
    }

    for (id, player) in state.players.iter().enumerate() {
        if !player.alive {
            continue
        }

        let segment_color = Color::hsl((id as f32)  * 30.0, 1.0, 0.8);
        let head_color = Color::hsl((id as f32) * 30.0, 1.0, 0.5);

        // commands
        //     .spawn(SpriteBundle {
        //         sprite: Sprite {
        //             color: head_color,
        //             ..default()
        //         },
        //         ..default()
        //     })
        //     .insert(SnakeHead)
        //     .insert(player.pos.clone())
        //     .insert(PlayerId(id))
        //     .insert(Size::square(0.8));

        let mut path_builder = PathBuilder::new();
        path_builder.move_to(Vec2::new(0., 0.));

        for (idx, mov) in player.moves.iter().enumerate() {
            // if idx == 0 {
            //     continue;
            // }

            //
            let x = convert(mov.x as f32, window.width() as f32, state.width as f32);
            let y = convert(mov.y as f32, window.height() as f32, state.height as f32);
            debug!("x {} y {}", x, y);
            path_builder.line_to(Vec2::new(x, y));

            // commands
            //     .spawn(SpriteBundle {
            //         sprite: Sprite {
            //             color: segment_color,
            //             ..default()
            //         },
            //         ..default()
            //     })
            //     .insert(SnakeHead)
            //     .insert(mov.clone())
            //     .insert(PreviousPos(player.moves.get(idx-1).unwrap().clone()))
            //     .insert(PlayerId(id))
            //     .insert(Size::square(0.8));
        }

        path_builder.close();
        let path = path_builder.build();
        commands.spawn((
            ShapeBundle {
                path,
                ..default()
            },
            Stroke::new(Color::BLACK, 10.0),
            Fill::color(Color::RED),
        ));
    }

    state_res.last_update = Instant::now();
}

const SNAKE_HEAD_COLOR: Color = Color::rgb(0.7, 0.7, 0.7);
const SNAKE_SEGMENT_COLOR: Color = Color::rgb(0.3, 0.3, 0.3);

#[derive(Component, Debug)]
struct SnakeHead;

fn size_scaling(
    foo: Res<ServerGameState>,
    windows: Query<&Window>,
    mut q: Query<(&Size, &mut Transform, &Position, &PreviousPos)>,
) {
    let state = foo.game_state.lock().unwrap();
    let window = windows.get_single().unwrap();
    for (sprite_size, mut transform, pos, prev) in q.iter_mut() {
        transform.scale = Vec3::new(
            sprite_size.width / state.width as f32 * window.width() as f32,
            sprite_size.height / state.height as f32 * window.height() as f32,
            1.0,
        );
    }
}

fn position_translation(
    foo: Res<ServerGameState>,
    windows: Query<&Window>,
    mut q: Query<(&Position, &mut Transform)>,
) {
    fn convert(pos: f32, bound_window: f32, bound_game: f32) -> f32 {
        let tile_size = bound_window / bound_game;
        pos / bound_game * bound_window - (bound_window / 2.) + (tile_size / 2.)
    }

    let state = foo.game_state.lock().unwrap();
    let window = windows.get_single().unwrap();
    for (pos, mut transform) in q.iter_mut() {
        transform.translation = Vec3::new(
            convert(pos.x as f32, window.width() as f32, state.width as f32),
            convert(pos.y as f32, window.height() as f32, state.height as f32),
            0.0,
        );
    }
}

// fn clear_dead(
//     mut commands: Commands,
//     mut version: ResMut<RenderedVersion>,
//     query: Query<Entity, With<SnakeHead>>,
//     mut foo: ResMut<ServerGameState>
// ) {
//     if version.state_version == version.cleared_version {
//         return;
//     }
//
//     version.cleared_version = version.state_version;
//
//     let state: MutexGuard<GameState> = foo.game_state.lock().unwrap();
//
//     //state.players.
//
//     for entity in query.iter() {
//
//
//         commands.entity(entity).despawn();
//     }
// }

fn render_everything(
    mut commands: Commands,
    mut foo: ResMut<ServerGameState>
) {
    // if version.cleared_version == version.rendered_version {
    //     return;
    // }
    //
    // version.rendered_version = version.cleared_version;

    let state: MutexGuard<GameState> = foo.game_state.lock().unwrap();
    println!("Rendering version {}", state.version);

    for (id, player) in state.players.iter().enumerate() {
        if !player.alive {
            continue
        }

        let segment_color = Color::hsl((id as f32)  * 30.0, 1.0, 0.8);
        let head_color = Color::hsl((id as f32) * 30.0, 1.0, 0.5);

        // commands
        //     .spawn(SpriteBundle {
        //         sprite: Sprite {
        //             color: head_color,
        //             ..default()
        //         },
        //         ..default()
        //     })
        //     .insert(SnakeHead)
        //     .insert(player.pos.clone())
        //     .insert(Size::square(0.8));

        // commands.spawn(MaterialMesh2dBundle {
        //     mesh: meshes
        //         .add(shape::Quad::new(Vec2::new(50., 100.)).into())
        //         .into(),
        //     material: materials.add(ColorMaterial::from(Color::LIME_GREEN)),
        //     transform: Transform::from_translation(Vec3::new(50., 0., 0.)),
        //     ..default()
        // });

        // for mov in &player.moves {
        //     if player.pos.eq(mov) {
        //         continue
        //     }
        //
        //     commands
        //         .spawn(SpriteBundle {
        //             sprite: Sprite {
        //                 color: segment_color,
        //                 ..default()
        //             },
        //             ..default()
        //         })
        //         .insert(SnakeHead)
        //         .insert(mov.clone())
        //         .insert(Size::square(0.8));
        // }
    }
}

