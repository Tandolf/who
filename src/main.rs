use std::{
    io::{self, Stdout},
    process,
};

use anyhow::{Context, Result};
use clap::Parser;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use dns::{message::Message, DeSerialize, Serialize};
use tokio::net::UdpSocket;

use crate::dns::Buffer;
use ratatui::{prelude::*, widgets::*};
mod dns;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    address: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    if args.address.is_empty() {
        eprintln!("please provide a valid address");
        process::exit(1);
    }

    // Implement validation, address max 255 ascii chars. And each part can be maximum 63.

    let sock = UdpSocket::bind("0.0.0.0:8080")
        .await
        .context("could not bind")?;

    let m = Message::single(args.address);
    let m = m.serialize().context("Failed to serialize request")?;

    let mut buffer = [0; 1024];
    let _len = sock.send_to(&m, "1.1.1.1:53").await?;
    let (_, _) = sock.recv_from(&mut buffer).await?;

    let mut buffer = Buffer {
        current: &buffer,
        source: &buffer,
    };

    let (_buffer, message) =
        Message::deserialize(&mut buffer).context("Failed to deserialize response")?;

    let mut terminal = setup_terminal().context("setup failed")?;
    terminal.draw(|f| render_app(f, &message))?;
    disable_raw_mode().context("failed to disable raw mode")?;
    let _ = terminal.show_cursor().context("unable to show cursor");

    Ok(())
}

fn setup_terminal() -> Result<Terminal<CrosstermBackend<Stdout>>> {
    let stdout = io::stdout();
    enable_raw_mode().context("failed to enable raw mode")?;
    let terminal = Terminal::with_options(
        CrosstermBackend::new(stdout),
        TerminalOptions {
            viewport: Viewport::Inline(15),
        },
    )?;
    Ok(terminal)
}

fn render_app(frame: &mut Frame, message: &Message) {
    let message = &message;
    let outer = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![Constraint::Percentage(70), Constraint::Percentage(30)])
        .split(frame.size());

    let inner = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![
            Constraint::Percentage(30),
            Constraint::Percentage(20),
            Constraint::Percentage(50),
        ])
        .split(outer[0]);
    // Header
    frame.render_widget(
        Paragraph::new(format!("{}", message.header))
            .block(Block::new().title("Header").borders(Borders::ALL)),
        inner[0],
    );

    // Question
    let row = Row::new(vec![
        Cell::from(message.question.qname.clone()),
        Cell::from(""),
        Cell::from(message.question.qclass.to_string()),
        Cell::from(message.question.qtype.to_string()),
    ]);

    let t = Table::new(vec![row])
        .block(Block::new().title("Message").borders(Borders::ALL))
        .widths(&[
            Constraint::Percentage(35),
            Constraint::Percentage(10),
            Constraint::Percentage(10),
            Constraint::Percentage(10),
        ]);

    frame.render_widget(t, inner[1]);

    // Records
    let record_rows = message.records.iter().map(|r| {
        let string_data = match &r.rdata {
            dns::record::RData::A(ip) => ip.to_string(),
            dns::record::RData::CNAME(cname) => cname.to_string(),
            dns::record::RData::TXT(txt) => txt.to_string(),
        };

        Row::new(vec![
            Cell::from(r.name.clone()),
            Cell::from(r.ttl.as_secs().to_string()),
            Cell::from(r.qclass.to_string()),
            Cell::from(r.qtype.to_string()),
            Cell::from(string_data),
        ])
    });

    let record_table = Table::new(record_rows)
        .block(Block::new().title("Records").borders(Borders::ALL))
        .widths(&[
            Constraint::Percentage(35),
            Constraint::Percentage(10),
            Constraint::Percentage(10),
            Constraint::Percentage(10),
            Constraint::Percentage(35),
        ]);
    frame.render_widget(record_table, inner[2]);
}
