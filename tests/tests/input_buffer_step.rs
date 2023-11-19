#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

use std::net::SocketAddr;
use std::str::FromStr;
use std::time::{Duration, Instant};

use bevy::log::LogPlugin;
use bevy::prelude::{
    App, Commands, EventReader, FixedUpdate, IntoSystemConfigs, PluginGroup, Real, Res, ResMut,
    Startup, Time,
};
use bevy::time::TimeUpdateStrategy;
use bevy::winit::WinitPlugin;
use bevy::{DefaultPlugins, MinimalPlugins};
use tracing::{debug, info};
use tracing_subscriber::fmt::format::FmtSpan;

use lightyear_shared::client::{Authentication, Client, ClientConfig, InputSystemSet};
use lightyear_shared::netcode::generate_key;
use lightyear_shared::plugin::events::InputEvent;
use lightyear_shared::plugin::sets::FixedUpdateSet;
use lightyear_shared::replication::Replicate;
use lightyear_shared::server::{NetcodeConfig, PingConfig, Server, ServerConfig};
use lightyear_shared::tick::Tick;
use lightyear_shared::{
    ChannelKind, ClientId, IoConfig, LinkConditionerConfig, MainSet, SharedConfig, TickConfig,
    TransportConfig,
};
use lightyear_tests::protocol::{protocol, Channel2, MyInput, MyProtocol};
use lightyear_tests::stepper::{BevyStepper, Step};
use lightyear_tests::tick_once;
use lightyear_tests::utils::{init_bevy_step, tick};

fn client_init(mut client: ResMut<Client<MyProtocol>>) {
    info!("Connecting to server");
    client.connect();
}

fn server_init(mut commands: Commands) {
    info!("Spawning entity on server");
    commands.spawn(Replicate {
        updates_channel: ChannelKind::of::<Channel2>(),
        ..Default::default()
    });
}

// System that runs every fixed timestep, and will add an input to the buffer
fn buffer_client_inputs(mut client: ResMut<Client<MyProtocol>>) {
    let tick = client.tick();
    client.add_input(MyInput(tick.0 as i16))
}

fn client_read_input(
    client: Res<Client<MyProtocol>>,
    mut input_reader: EventReader<InputEvent<MyInput>>,
) {
    for input in input_reader.read() {
        info!(
            "Client has input {:?} at tick {:?}",
            input.input(),
            client.tick()
        );
    }
}

fn server_read_input(
    // TODO: maybe put the tick in a separate resource? it lowers parallelism to have to fetch the entire server just to get the tick..
    server: Res<Server<MyProtocol>>,
    mut input_reader: EventReader<InputEvent<MyInput, ClientId>>,
) {
    let tick = server.tick();
    for input in input_reader.read() {
        if input.input().is_some() {
            info!(
                "Server received input {:?} from client {:?} at tick {:?}",
                input.input(),
                input.context(),
                tick
            );
        }
    }
}

#[test]
fn test_bevy_step() -> anyhow::Result<()> {
    let frame_duration = Duration::from_secs_f32(1.0 / 60.0);
    let tick_duration = Duration::from_millis(10);
    let shared_config = SharedConfig {
        enable_replication: false,
        tick: TickConfig::new(tick_duration),
        ..Default::default()
    };
    let link_conditioner = LinkConditionerConfig {
        incoming_latency: 20,
        incoming_jitter: 0,
        incoming_loss: 0.0,
    };
    let mut stepper = BevyStepper::new(shared_config, link_conditioner, frame_duration);

    // add systems
    stepper.client_app.add_systems(Startup, client_init);
    stepper.server_app.add_systems(Startup, server_init);
    stepper.client_app.add_systems(
        FixedUpdate,
        buffer_client_inputs.in_set(InputSystemSet::BufferInputs),
    );
    stepper
        .client_app
        .add_systems(FixedUpdate, client_read_input.in_set(FixedUpdateSet::Main));
    stepper
        .server_app
        .add_systems(FixedUpdate, server_read_input.in_set(FixedUpdateSet::Main));

    // tick a bit, and check the input buffer received on server
    for i in 0..200 {
        stepper.frame_step();
    }

    // TODO: add asserts? at least we correctly receive inputs!

    // TODO:
    //  -Sometimes, the client's InputMessage has some absent inputs in the middle for some reason ??
    //     - not sure if it still happens
    //  -check on client behaves during rollback (need to use rollback tick)

    Ok(())
}