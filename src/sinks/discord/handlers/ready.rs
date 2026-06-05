use serenity::all::{Context, Ready};

pub async fn ready(_ctx: Context, ready: Ready) {
    println!("Logged in as {} (id={})", ready.user.name, ready.user.id);
}
