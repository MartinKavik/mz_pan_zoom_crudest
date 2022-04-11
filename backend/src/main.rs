use moon::*;

async fn frontend() -> Frontend {
    Frontend::new()
        .title("Pan & Zoom Test")
        .append_to_head(r#"<link href="/_api/public/css/custom.css" rel="stylesheet"/>"#)
}

async fn up_msg_handler(_: UpMsgRequest<()>) {}

#[moon::main]
async fn main() -> std::io::Result<()> {
    start(frontend, up_msg_handler, |_| {}).await
}
