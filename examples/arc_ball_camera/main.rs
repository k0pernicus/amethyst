//! Demonstrates how to use the fly camera

use amethyst::{
    assets::{PrefabLoader, PrefabLoaderSystem, RonFormat},
    controls::{ArcBallControlBundle, ArcBallControlTag},
    core::{
        shrev::{EventChannel, ReaderId},
        transform::{Transform, TransformBundle},
    },
    ecs::prelude::{Join, Read, ReadStorage, System, SystemData, World, WorldExt, WriteStorage},
    input::{
        is_key_down, InputBundle, InputEvent, ScrollDirection, StringBindings, VirtualKeyCode,
    },
    prelude::*,
    renderer::{
        palette::Srgb,
        plugins::{RenderShaded3D, RenderSkybox, RenderToWindow},
        rendy::mesh::{Normal, Position, TexCoord},
        types::DefaultBackend,
        RenderingBundle,
    },
    utils::{application_root_dir, scene::BasicScenePrefab},
    Error,
};

type MyPrefabData = BasicScenePrefab<(Vec<Position>, Vec<Normal>, Vec<TexCoord>), f32>;

struct ExampleState;

impl SimpleState for ExampleState {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let prefab_handle = data.world.exec(|loader: PrefabLoader<'_, MyPrefabData>| {
            loader.load("prefab/arc_ball_camera.ron", RonFormat, ())
        });
        data.world.create_entity().with(prefab_handle).build();
    }

    fn handle_event(
        &mut self,
        _: StateData<'_, GameData<'_, '_>>,
        event: StateEvent,
    ) -> SimpleTrans {
        if let StateEvent::Window(event) = event {
            if is_key_down(&event, VirtualKeyCode::Escape) {
                Trans::Quit
            } else {
                Trans::None
            }
        } else {
            Trans::None
        }
    }
}

struct CameraDistanceSystem {
    event_reader: ReaderId<InputEvent<StringBindings>>,
}

impl CameraDistanceSystem {
    pub fn new(mut world: &mut World) -> Self {
        <Self as System<'_>>::SystemData::setup(&mut world);
        let event_reader = world
            .fetch_mut::<EventChannel<InputEvent<StringBindings>>>()
            .register_reader();

        CameraDistanceSystem { event_reader }
    }
}

impl<'a> System<'a> for CameraDistanceSystem {
    type SystemData = (
        Read<'a, EventChannel<InputEvent<StringBindings>>>,
        ReadStorage<'a, Transform>,
        WriteStorage<'a, ArcBallControlTag>,
    );

    fn run(&mut self, (events, transforms, mut tags): Self::SystemData) {
        for event in events.read(&mut self.event_reader) {
            if let InputEvent::MouseWheelMoved(direction) = *event {
                match direction {
                    ScrollDirection::ScrollUp => {
                        for (_, tag) in (&transforms, &mut tags).join() {
                            tag.distance *= 0.9;
                        }
                    }
                    ScrollDirection::ScrollDown => {
                        for (_, tag) in (&transforms, &mut tags).join() {
                            tag.distance *= 1.1;
                        }
                    }
                    _ => (),
                }
            }
        }
    }
}

fn main() -> Result<(), Error> {
    amethyst::start_logger(Default::default());

    let app_root = application_root_dir()?;

    let assets_directory = app_root.join("examples/assets");
    let display_config_path = app_root.join("examples/arc_ball_camera/config/display.ron");

    let key_bindings_path = app_root.join("examples/arc_ball_camera/config/input.ron");

    let mut world = World::new();

    let game_data = GameDataBuilder::default()
        .with(PrefabLoaderSystem::<MyPrefabData>::new(&mut world), "", &[])
        .with_bundle(
            &mut world,
            RenderingBundle::<DefaultBackend>::new()
                .with_plugin(RenderToWindow::from_config_path(display_config_path))
                .with_plugin(RenderShaded3D::default())
                .with_plugin(RenderSkybox::with_colors(
                    Srgb::new(0.82, 0.51, 0.50),
                    Srgb::new(0.18, 0.11, 0.85),
                )),
        )?
        .with_bundle(&mut world, TransformBundle::new().with_dep(&[]))?
        .with_bundle(
            &mut world,
            InputBundle::<StringBindings>::new().with_bindings_from_file(&key_bindings_path)?,
        )?
        .with_bundle(&mut world, ArcBallControlBundle::<StringBindings>::new())?
        .with(
            CameraDistanceSystem::new(&mut world),
            "camera_distance_system",
            &["input_system"],
        );

    let mut game = Application::build(assets_directory, ExampleState, world)?.build(game_data)?;
    game.run();
    Ok(())
}
