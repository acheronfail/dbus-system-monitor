use std::time::Duration;

use dbus::blocking::Connection;
use dbus::channel::MatchingReceiver;
use dbus::message::MatchRule;
use dbus::Message;

// This programs implements the equivalent of running the "dbus-monitor" tool
fn main() {
    // First open up a connection to the session bus.
    let conn = Connection::new_system().expect("D-Bus connection failed");
    eprintln!("name: {:?}\n", conn.unique_name());

    // Second create a rule to match messages we want to receive; in this example we add no
    // further requirements, so all messages will match
    let mut rule = MatchRule::new()
        .with_type(dbus::MessageType::Signal)
        .with_interface("org.freedesktop.NetworkManager");

    // Try matching using new scheme
    let proxy = conn.with_proxy(
        "org.freedesktop.DBus",
        "/org/freedesktop/DBus",
        Duration::from_millis(5000),
    );
    let result: Result<(), dbus::Error> = proxy.method_call(
        "org.freedesktop.DBus.Monitoring",
        "BecomeMonitor",
        (vec![rule.match_str()], 0u32),
    );

    if result.is_ok() {
        // Start matching using new scheme
        conn.start_receive(
            rule,
            Box::new(|msg, _| {
                handle_message(&msg);
                true
            }),
        );
    } else {
        eprintln!("falling back to eavesdrop: {}\n", result.unwrap_err());
        // Start matching using old scheme
        rule.eavesdrop = true; // this lets us eavesdrop on *all* session messages, not just ours
        eprintln!("{}\n", rule.match_str());

        conn.add_match(rule, |_: (), _, msg| {
            handle_message(&msg);
            true
        })
        .expect("add_match failed");
    }

    // Loop and print out all messages received (using handle_message()) as they come.
    // Some can be quite large, e.g. if they contain embedded images..
    loop {
        conn.process(Duration::from_millis(1000)).unwrap();
    }
}

fn handle_message(msg: &Message) {
    println!("Got message: {:?}", msg);
}
