#[macro_use]
extern crate itertools;

use std::collections::HashMap;
use std::collections::BTreeSet;
use std::fs;

#[derive (Debug)]
struct Point {
    x: u32,
    y: u32,
}

#[derive (Ord, PartialOrd, Eq, PartialEq)]
struct Edge {
    source: u32,
    target: u32,
}

struct Node {
    column: u8,
    name: String,
}

#[derive (Debug)]
struct Rect {
    height: u32,
    width: u32,
    top_left: Point,
}

fn main() {
    let default_height = 40;
    let default_width = 120;
    let default_gap = 25;

    let source_index = (0..48).collect::<Vec<_>>();
    let column_chooser = |i| i % 7 as u8;

    let rects: Vec<_> = source_index
        .iter()
        .map(|i| Node {
            column: column_chooser(i),
            name: "Test".to_owned(),
        })
        .collect();

    let edges: BTreeSet<_> = {
        let column_by_index: HashMap<u8, u8> = source_index
            .iter()
            .map(|i| (*i as u8, column_chooser(i)))
            .collect();

        iproduct!(source_index.clone(), source_index.clone())
            .filter_map(|(start, end)| {
                let start_col = column_by_index.get(&start)?;
                let end_col = column_by_index.get(&end)?;
                if start != end && (*start_col as i32 - *end_col as i32).abs() <= 1 {
                    Some(Edge {
                        source: start as u32,
                        target: end as u32,
                    })
                } else {
                    None
                }
            })
            .collect()
    };

    let mut current_y: HashMap<u8, u32> = HashMap::new();
    let mut max_height = 0u32;
    let mut max_width = 0u32;

    let placed_recs: Vec<_> = rects
        .iter()
        .map(|node| {
            let next = Rect {
                height: default_height,
                width: default_width,
                top_left: Point {
                    x: (node.column as u32) * (default_width + default_gap),
                    y: current_y
                        .get(&node.column)
                        .map(|y| y + default_height + default_gap)
                        .unwrap_or(0u32),
                },
            };

            max_height = max_height.max(next.top_left.y + next.height);
            max_width = max_width.max(next.top_left.x + next.width);

            current_y.insert(node.column, next.top_left.y);

            next
        })
        .collect();

    let rect_str = placed_recs
        .iter()
        .map(|rect| {
            format!(
                r#"<rect x="{}" y="{}" height="{}" width="{}" />"#,
                rect.top_left.x, rect.top_left.y, rect.height, rect.width
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    let edges_str = edges
        .iter()
        .filter_map(|Edge { source, target }| {
            let start = placed_recs.get(*source as usize)?;
            let end = placed_recs.get(*target as usize)?;

            let next = format!(
                r#"<path d="
                    M {} {}
                    L {} {}" />"#,
                start.top_left.x,
                start.top_left.y,
                end.top_left.x,
                end.top_left.y
            );

            Some(next)
        })
        .collect::<Vec<_>>()
        .join("\n");

    //r#"<path d="M10 10 H 90 V 90 H 10 L 10 10" />"#;

    let svg = format!(
        r#"
    <html>
    <head>
      <meta charset="utf-8"></meta>
      <style type="text/css" >
        rect {{
          stroke: #006600;
          fill:   #00cc00;
        }}

        path {{
          stroke-width: 2px;
          stroke: purple;
          fill:  none;
        }}
      </style>
    </head>
    <body>
    <svg height="{}" width="{}">
     {}
     {}
     </svg>
     </body>"#,
        max_height, max_width, edges_str, rect_str,
    );

    fs::write("index.html", svg).expect("Could not write svg out.")
}
