use bytes::Bytes;
use futures::Stream;
use rweb::{get, post, sse::Event, Filter, Rejection, Reply};
use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, Mutex,
    },
};
use tokio::sync::{mpsc, oneshot};
use tokio_stream::{wrappers::UnboundedReceiverStream, StreamExt};

/// Our global unique user id counter.
static NEXT_USER_ID: AtomicUsize = AtomicUsize::new(1);

/// Message variants.
enum Message {
    UserId(usize),
    Reply(String),
}

#[derive(Debug)]
struct NotUtf8;
impl rweb::reject::Reject for NotUtf8 {}

/// Our state of currently connected users.
///
/// - Key is their id
/// - Value is a sender of `Message`
///
/// TODO(kdy1): .and(rweb::body::content_length_limit(500))
type Users = Arc<Mutex<HashMap<usize, mpsc::UnboundedSender<Message>>>>;

#[post("/chat/{my_id}")]
async fn send_chat(
    my_id: usize,
    #[body] msg: Bytes,
    #[data] users: Users,
) -> Result<impl Reply, Rejection> {
    let msg = std::str::from_utf8(&msg)
        .map(String::from)
        .map_err(|_e| rweb::reject::custom(NotUtf8))?;

    user_message(my_id, msg, &users);
    Ok(rweb::reply())
}

#[get("/chat")]
fn recv_chat(#[data] users: Users) -> impl Reply {
    // reply using server-sent events
    let stream = user_connected(users);
    rweb::sse::reply(rweb::sse::keep_alive().stream(stream))
}

#[get("/")]
fn index() -> impl Reply {
    rweb::http::Response::builder()
        .header("content-type", "text/html; charset=utf-8")
        .body(INDEX_HTML)
}

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    // Keep track of all connected users, key is usize, value
    // is an event stream sender.
    let users = Arc::new(Mutex::new(HashMap::new()));

    // POST /chat -> send message
    let chat_send = send_chat(users.clone());

    // GET /chat -> messages stream
    let chat_recv = recv_chat(users.clone());

    let routes = index().or(chat_recv).or(chat_send);

    rweb::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}

fn user_connected(users: Users) -> impl Stream<Item = Result<Event, rweb::Error>> + Send + 'static {
    // Use a counter to assign a new unique ID for this user.
    let my_id = NEXT_USER_ID.fetch_add(1, Ordering::Relaxed);

    eprintln!("new chat user: {}", my_id);

    // Use an unbounded channel to handle buffering and flushing of messages
    // to the event source...
    let (tx, rx) = mpsc::unbounded_channel();

    match tx.send(Message::UserId(my_id)) {
        Ok(()) => (),
        Err(_disconnected) => {
            // The tx is disconnected, our `user_disconnected` code
            // should be happening in another task, nothing more to
            // do here.
        }
    }

    // Make an extra clone of users list to give to our disconnection handler...
    let users2 = users.clone();

    // Save the sender in our list of connected users.
    users.lock().unwrap().insert(my_id, tx);

    // Create channel to track disconnecting the receiver side of events.
    // This is little bit tricky.
    let (mut dtx, mut drx) = oneshot::channel::<()>();

    // When `drx` will dropped then `dtx` will be canceled.
    // We can track it to make sure when the user leaves chat.
    tokio::task::spawn(async move {
        dtx.closed().await;
        drx.close();
        user_disconnected(my_id, &users2);
    });

    // Convert messages into Server-Sent Events and return resulting stream.
    UnboundedReceiverStream::new(rx).map(|msg| match msg {
        Message::UserId(my_id) => Ok(rweb::sse::Event::default()
            .event("user")
            .data(my_id.to_string())),
        Message::Reply(reply) => Ok(rweb::sse::Event::default().data(reply)),
    })
}

fn user_message(my_id: usize, msg: String, users: &Users) {
    let new_msg = format!("<User#{}>: {}", my_id, msg);

    // New message from this user, send it to everyone else (except same uid)...
    //
    // We use `retain` instead of a for loop so that we can reap any user that
    // appears to have disconnected.
    for (&uid, tx) in users.lock().unwrap().iter_mut() {
        if my_id != uid {
            match tx.send(Message::Reply(new_msg.clone())) {
                Ok(()) => (),
                Err(_disconnected) => {
                    // The tx is disconnected, our `user_disconnected` code
                    // should be happening in another task, nothing more to
                    // do here.
                }
            }
        }
    }
}

fn user_disconnected(my_id: usize, users: &Users) {
    eprintln!("good bye user: {}", my_id);

    // Stream closed up, so remove from the user list
    users.lock().unwrap().remove(&my_id);
}

static INDEX_HTML: &str = r#"
<!DOCTYPE html>
<html>
    <head>
        <title>Warp Chat</title>
    </head>
    <body>
        <h1>warp chat</h1>
        <div id="chat">
            <p><em>Connecting...</em></p>
        </div>
        <input type="text" id="text" />
        <button type="button" id="send">Send</button>
        <script type="text/javascript">
        var uri = 'http://' + location.host + '/chat';
        var sse = new EventSource(uri);
        function message(data) {
            var line = document.createElement('p');
            line.innerText = data;
            chat.appendChild(line);
        }
        sse.onopen = function() {
            chat.innerHTML = "<p><em>Connected!</em></p>";
        }
        var user_id;
        sse.addEventListener("user", function(msg) {
            user_id = msg.data;
        });
        sse.onmessage = function(msg) {
            message(msg.data);
        };
        send.onclick = function() {
            var msg = text.value;
            var xhr = new XMLHttpRequest();
            xhr.open("POST", uri + '/' + user_id, true);
            xhr.send(msg);
            text.value = '';
            message('<You>: ' + msg);
        };
        </script>
    </body>
</html>
"#;
