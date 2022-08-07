use bevy::prelude::*;

use bevy_ui_navigation::components::FocusableButtonBundle;
use bevy_ui_navigation::prelude::{
    DefaultNavigationPlugins, FocusState, Focusable, MenuBuilder, MenuSetting, NavRequestSystem,
};

/// THE ULTIMATE MENU DEMONSTRATION
///
/// This is an unrealistic menu demonstrating tabbed navigation, focus memory
/// and navigation hierarchy traversal. It is similar to your classical RPG
/// menu, with the significant difference that **all tabs are shown at the same
/// time on screen** rather than hidden and shown as the tabs are selected.
///
/// The use of macros is not _needed_ but extremely useful. Removes the noise
/// from the ui declaration and helps focus the example on the important stuff,
/// not the UI building boilerplate.
///
/// Use `Q` and `E` to navigate tabs, use `WASD` for moving within containers,
/// `ENTER` and `BACKSPACE` for going down/up the hierarchy.
///
/// Navigation also works with controller
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(DefaultNavigationPlugins)
        .add_startup_system(setup)
        // IMPORTANT: setting the button appearance update system after the
        // NavRequestSystem makes everything much snappier, highly recommended.
        .add_system(button_system.after(NavRequestSystem))
        .add_system(block_some_focusables.before(NavRequestSystem))
        .run();
}

fn block_some_focusables(
    mut focusables: Query<&mut Focusable>,
    mut blocked_index: Local<usize>,
    time: Res<Time>,
) {
    let delta = time.delta_seconds_f64();
    let current_time = time.seconds_since_startup();
    let at_interval = |t: f64| current_time % t < delta;

    if at_interval(3.0) {
        let mut skipped = focusables.iter_mut().skip(*blocked_index);
        if skipped.len() == 0 {
            *blocked_index = 0;
        }
        *blocked_index += 3;
        for mut to_unblock in skipped.by_ref().take(3) {
            to_unblock.unblock();
        }
        for mut to_block in skipped.take(3) {
            to_block.block();
        }
    }
}

fn button_system(mut interaction_query: Query<(&Focusable, &mut UiColor), Changed<Focusable>>) {
    for (focus, mut material) in interaction_query.iter_mut() {
        let color = match focus.state() {
            FocusState::Focused => Color::ORANGE_RED,
            FocusState::Active => Color::GOLD,
            FocusState::Prioritized => Color::GRAY,
            FocusState::Inert => Color::DARK_GRAY,
            FocusState::Blocked => Color::ANTIQUE_WHITE,
        };
        *material = color.into();
    }
}

fn setup(mut commands: Commands) {
    use FlexDirection::{ColumnReverse, Row};
    use FlexWrap::Wrap;
    use JustifyContent::{FlexStart, SpaceBetween};
    // ui camera
    commands.spawn_bundle(Camera2dBundle::default());

    let red: UiColor = Color::RED.into();
    let blue: UiColor = Color::BLUE.into();
    let green: UiColor = Color::GREEN.into();
    let gray: UiColor = Color::rgba(0.9, 0.9, 0.9, 0.3).into();
    let transparent: UiColor = Color::NONE.into();

    let pct = Val::Percent;
    let px = Val::Px;
    let vertical = NodeBundle {
        style: Style {
            flex_direction: ColumnReverse,
            size: Size::new(pct(100.0), pct(100.0)),
            margin: UiRect::all(px(2.0)),
            ..default()
        },
        color: transparent,
        ..default()
    };
    let horizontal = NodeBundle {
        style: Style {
            flex_direction: Row,
            size: Size::new(pct(100.0), pct(100.0)),
            justify_content: SpaceBetween,
            margin: UiRect::all(px(2.0)),
            ..default()
        },
        color: transparent,
        ..default()
    };
    let square = FocusableButtonBundle::from(ButtonBundle {
        style: Style {
            size: Size::new(px(40.0), px(40.0)),
            margin: UiRect::all(px(2.0)),
            ..default()
        },
        ..default()
    });
    let long = FocusableButtonBundle::from(ButtonBundle {
        style: Style {
            size: Size::new(pct(100.0), px(40.0)),
            margin: UiRect::all(px(2.0)),
            ..default()
        },
        ..default()
    });
    let tab_square = FocusableButtonBundle::from(ButtonBundle {
        style: Style {
            size: Size::new(px(100.0), px(40.0)),
            margin: UiRect {
                left: px(30.0),
                right: px(30.0),
                top: px(0.0),
                bottom: px(0.0),
            },
            ..default()
        },
        ..default()
    });
    let column_box = NodeBundle {
        style: Style {
            flex_direction: Row,
            flex_basis: pct(90.0),
            size: Size::new(pct(100.0), pct(90.0)),
            padding: UiRect::all(px(30.0)),
            ..default()
        },
        ..default()
    };
    let column = NodeBundle {
        style: Style {
            flex_direction: ColumnReverse,
            size: Size::new(pct(33.0), pct(100.0)),
            padding: UiRect::all(px(10.0)),
            margin: UiRect {
                left: px(5.0),
                right: px(5.0),
                top: px(0.0),
                bottom: px(0.0),
            },
            ..default()
        },
        ..default()
    };
    let colored_square = NodeBundle {
        style: Style {
            size: Size::new(pct(100.0), pct(100.0)),
            ..default()
        },
        color: Color::rgb(1.0, 0.3, 0.9).into(),
        ..default()
    };

    let menu = |name| (MenuSetting::new(), MenuBuilder::from_named(name));
    let cycle_menu = |name| (MenuSetting::new().wrapping(), MenuBuilder::from_named(name));
    let named = Name::new;

    // Note that bevy's native UI library IS NOT NICE TO WORK WITH. I
    // personally use `build_ui` from `bevy_ui_build_macros`, but for the sake
    // of comprehension, I use the native way of creating a UI here.
    //
    // Pay attention to calls to `menu("id")`, `cycle_menu("id"), `named`, and
    // `MenuSetting::root()`. You'll notice we use `Name` to give a sort of
    // identifier to our focusables so that they are refereable by `MenuSetting`s
    // afterward.
    commands
        .spawn_bundle(vertical.clone())
        .insert(Style {
            size: Size::new(pct(100.0), pct(100.0)),
            ..vertical.style.clone()
        })
        // The tab menu should be navigated with `NavRequest::ScopeMove` hence the `.scope()`
        //             vvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvv
        .insert_bundle((MenuSetting::new().wrapping().scope(), MenuBuilder::Root))
        .with_children(|cmds| {
            cmds.spawn_bundle(horizontal.clone())
                .insert(Style {
                    justify_content: FlexStart,
                    flex_basis: pct(10.0),
                    ..horizontal.style.clone()
                })
                .with_children(|cmds| {
                    // adding a `Name` component let us refer to those entities
                    // later without having to store their `Entity` ids anywhere.
                    cmds.spawn_bundle(tab_square.clone()).insert(named("red"));
                    cmds.spawn_bundle(tab_square.clone()).insert(named("green"));
                    cmds.spawn_bundle(tab_square).insert(named("blue"));
                });
            cmds.spawn_bundle(column_box).with_children(|cmds| {
                cmds.spawn_bundle(column.clone())
                    // refers to the "red" `tab_square`
                    //                 vvvvvvvvvvv
                    .insert_bundle(menu("red"))
                    .insert(red)
                    .with_children(|cmds| {
                        cmds.spawn_bundle(vertical.clone()).with_children(|cmds| {
                            cmds.spawn_bundle(long.clone()).insert(named("select1"));
                            cmds.spawn_bundle(long.clone()).insert(named("select2"));
                        });
                        cmds.spawn_bundle(horizontal.clone())
                            .insert(Style {
                                flex_wrap: Wrap,
                                ..horizontal.style.clone()
                            })
                            .insert_bundle(cycle_menu("select1"))
                            .insert(gray)
                            .with_children(|cmds| {
                                for _ in 0..20 {
                                    cmds.spawn_bundle(square.clone());
                                }
                            });
                        cmds.spawn_bundle(horizontal.clone())
                            .insert(Style {
                                flex_wrap: Wrap,
                                ..horizontal.style.clone()
                            })
                            .insert_bundle(cycle_menu("select2"))
                            .insert(gray)
                            .with_children(|cmds| {
                                for _ in 0..8 {
                                    cmds.spawn_bundle(square.clone());
                                }
                            });
                    });
                cmds.spawn_bundle(column.clone())
                    // refers to the "green" `tab_square`
                    //             vvvvvvvvvvvvv
                    .insert_bundle(menu("green"))
                    .insert(green)
                    .with_children(|cmds| {
                        for i in 0..8 {
                            let name = format!("green_{i}");
                            let child_bundle = if i % 2 == 0 {
                                (
                                    MenuSetting::new().wrapping(),
                                    MenuBuilder::from_named(name.clone()),
                                )
                            } else {
                                (MenuSetting::new(), MenuBuilder::from_named(name.clone()))
                            };
                            cmds.spawn_bundle(horizontal.clone()).with_children(|cmds| {
                                cmds.spawn_bundle(long.clone()).insert(Name::new(name));
                                cmds.spawn_bundle(horizontal.clone())
                                    .insert_bundle(child_bundle)
                                    .insert(gray)
                                    .with_children(|cmds| {
                                        for _ in 0..i % 6 + 1 {
                                            cmds.spawn_bundle(square.clone());
                                        }
                                    });
                            });
                        }
                    });
                cmds.spawn_bundle(column.clone())
                    // refers to the "blue" `tab_square`
                    //             vvvvvvvvvvvv
                    .insert_bundle(menu("blue"))
                    .insert(blue)
                    .with_children(|cmds| {
                        cmds.spawn_bundle(vertical.clone()).with_children(|cmds| {
                            cmds.spawn_bundle(vertical).with_children(|cmds| {
                                for _ in 0..6 {
                                    cmds.spawn_bundle(long.clone());
                                }
                            });
                            cmds.spawn_bundle(colored_square);
                        });
                    });
            });
        });
}
