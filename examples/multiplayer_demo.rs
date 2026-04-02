//! Example: Create a server and client NetState, send messages, show state sync.

use kiran::net::{EntityState, InputMessage, NetMessage, NetState, StateSnapshot};

fn main() {
    // Create server and client
    let mut server = NetState::server("server-01");
    let mut client = NetState::client("player-42");

    server.add_peer("player-42");
    client.add_peer("server-01");

    println!(
        "Server: node_id={}, role={:?}, peers={}",
        server.node_id,
        server.role,
        server.peer_count()
    );
    println!(
        "Client: node_id={}, role={:?}, peers={}",
        client.node_id,
        client.role,
        client.peer_count()
    );

    // Client sends input to server
    client.send(NetMessage::InputReplication(InputMessage {
        node_id: "player-42".into(),
        tick: 1,
        payload: r#"{"move": [1.0, 0.0]}"#.into(),
    }));

    // Drain client outbox and deliver to server inbox
    let outgoing = client.drain_outbox();
    println!("\nClient sent {} message(s)", outgoing.len());
    for msg in outgoing {
        server.receive(msg);
    }

    // Server processes inbox
    let incoming = server.drain_inbox();
    for msg in &incoming {
        if let NetMessage::InputReplication(input) = msg {
            println!(
                "Server received input from {} at tick {}: {}",
                input.node_id, input.tick, input.payload
            );
        }
    }

    // Server sends a state snapshot back
    server.advance_tick();
    server.send(NetMessage::StateSnapshot(StateSnapshot {
        tick: server.tick,
        entities: vec![
            EntityState {
                entity_id: 1,
                position: [5.0, 0.0, 3.0],
                owner: Some("player-42".into()),
            },
            EntityState {
                entity_id: 2,
                position: [0.0, 10.0, 0.0],
                owner: None,
            },
        ],
    }));

    let snapshot_msgs = server.drain_outbox();
    println!("\nServer sent snapshot at tick {}", server.tick);
    for msg in snapshot_msgs {
        client.receive(msg);
    }

    // Client processes the snapshot
    let client_inbox = client.drain_inbox();
    for msg in &client_inbox {
        if let NetMessage::StateSnapshot(snap) = msg {
            println!(
                "Client received snapshot (tick={}, {} entities):",
                snap.tick,
                snap.entities.len()
            );
            for e in &snap.entities {
                let owner = e.owner.as_deref().unwrap_or("(none)");
                println!(
                    "  entity {} @ ({:.1}, {:.1}, {:.1}) owner={owner}",
                    e.entity_id, e.position[0], e.position[1], e.position[2]
                );
            }
        }
    }
}
