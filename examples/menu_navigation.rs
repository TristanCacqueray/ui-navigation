use bevy::prelude::*;

use bevy_ui_navigation::prelude::{
    DefaultNavigationPlugins, FocusState, Focusable, MenuBuilder, MenuSetting, NavEvent,
    NavRequest, NavRequestSystem,
};

/// This example demonstrates a more complex menu system where you navigate
/// through menus and go to submenus using the `Action` and `Cancel`
/// (`ENTER` and `BACKSPACE` on keyboard) requests.
///
/// This introduces the concept of "active" and "dormant" focusable elements.
///
/// The focus goes back to active elements from the parent menu if you request
/// `Cancel` in a given submenu.
///
/// The focus goes back to the child menu's dormant element if you request
/// `Action` while the parent menu's corresponding `Focusable` is focused.
///
/// To navigate to the right column, move focus to the button with the right arrow
/// and press `ENTER`, to navigate to the left, press `BACKSPACE`. Notice how
/// going back to an already explored menu sets the focused element to the last
/// focused one.
///
/// This example also demonstrates the `NavRequest::FocusOn` request. When
/// `ENTER` is pressed when a green circle button is focused, it sends the
/// `FocusOn` request with a first row button as target.
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(DefaultNavigationPlugins)
        .init_resource::<Materials>()
        .insert_resource(Gameui::new())
        .add_startup_system(setup)
        .add_system(button_system.after(NavRequestSystem))
        .add_system(handle_nav_events.after(NavRequestSystem))
        .run();
}

#[derive(Resource)]
struct Gameui {
    from: Vec<Entity>,
    to: Entity,
}
impl Gameui {
    pub fn new() -> Self {
        Self {
            from: Vec::new(),
            to: Entity::from_raw(1),
        }
    }
}

#[derive(Resource)]
struct Materials {
    background: Color,
    rarrow: UiImage,
    circle: UiImage,
}

impl FromWorld for Materials {
    fn from_world(world: &mut World) -> Self {
        let assets = world.get_resource::<AssetServer>().unwrap();
        Materials {
            background: Color::BLACK,
            rarrow: assets.load("rarrow.png").into(),
            circle: assets.load("green_circle.png").into(),
        }
    }
}

fn button_system(
    mut interaction_query: Query<(&Focusable, &mut BackgroundColor), Changed<Focusable>>,
) {
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

fn handle_nav_events(
    mut events: EventReader<NavEvent>,
    mut requests: EventWriter<NavRequest>,
    game: Res<Gameui>,
) {
    use NavRequest::Action;
    for event in events.iter() {
        if let NavEvent::FocusChanged { from, to } = &event {
            println!("----------\nfrom: {:?}\n  to: {:?}", from, to);
        }
        match event {
            NavEvent::NoChanges {
                from,
                request: Action,
            } if game.from.contains(from.first()) => requests.send(NavRequest::FocusOn(game.to)),
            _ => {}
        }
    }
}

fn menu(materials: &Materials) -> NodeBundle {
    let size_fn = |width, height| Size::new(Val::Percent(width), Val::Percent(height));
    let style = Style {
        size: size_fn(20.0, 95.0),
        flex_direction: FlexDirection::Column,
        flex_wrap: FlexWrap::Wrap,
        justify_content: JustifyContent::Center,
        align_content: AlignContent::Stretch,
        ..Default::default()
    };
    NodeBundle {
        style,
        background_color: materials.background.into(),
        ..Default::default()
    }
}
fn setup(mut commands: Commands, materials: Res<Materials>, mut game: ResMut<Gameui>) {
    // ui camera
    commands.spawn(Camera2dBundle::default());

    let size_fn = |width, height| Size::new(Val::Percent(width), Val::Percent(height));
    let style = Style {
        position_type: PositionType::Absolute,
        flex_direction: FlexDirection::Row,
        size: size_fn(100.0, 100.0),
        ..Default::default()
    };
    let bundle = NodeBundle {
        style,
        ..Default::default()
    };
    let image_style = Style {
        size: size_fn(100.0, 100.0),
        ..Default::default()
    };
    let rarrow = ImageBundle {
        style: image_style.clone(),
        image: materials.rarrow.clone(),
        ..Default::default()
    };
    let circle = ImageBundle {
        style: image_style,
        image: materials.circle.clone(),
        ..Default::default()
    };

    commands.spawn(bundle).with_children(|commands| {
        let mut next_menu_button: Option<Entity> = None;
        for j in 0..5 {
            commands
                .spawn((
                    menu(&materials),
                    // Note: when next_menu_button is None,
                    // `with_parent(next_menu_button)` represents the root menu
                    MenuSetting::new().wrapping(),
                    MenuBuilder::from(next_menu_button),
                ))
                .with_children(|commands| {
                    for i in 0..4 {
                        let mut button = commands.spawn(button());
                        button.insert(Focusable::default());
                        if j == 0 && i == 3 {
                            game.to = button.id();
                        }
                        if j == i {
                            button.with_children(|commands| {
                                commands.spawn(rarrow.clone());
                            });
                            next_menu_button = Some(button.id());
                        }
                        if j == 3 && i == 1 {
                            button.insert(Focusable::cancel()).with_children(|cmds| {
                                cmds.spawn(circle.clone());
                            });
                        }
                        if j == 2 && i == 1 {
                            button.insert(Focusable::new().blocked());
                        }
                        if j == 4 {
                            let to_add = button
                                .with_children(|commands| {
                                    commands.spawn(circle.clone());
                                })
                                .id();
                            game.from.push(to_add);
                        }
                    }
                });
        }
    });
}
fn button() -> ButtonBundle {
    let size_fn = |width, height| Size::new(Val::Percent(width), Val::Percent(height));
    let size = size_fn(95.0, 12.0);
    let style = Style {
        size,
        margin: UiRect::all(Val::Percent(3.0)),
        ..Default::default()
    };
    ButtonBundle {
        style,
        ..Default::default()
    }
}
