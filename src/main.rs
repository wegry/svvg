#[macro_use]
extern crate itertools;

use std::collections::BTreeSet;
use std::collections::HashMap;
use std::fs;
use std::ops::{Add, Sub};

#[derive(Clone, Debug)]
struct Point {
    x: u32,
    y: u32,
}

impl Point {
    fn svg_path_point(&self) -> String {
        format!("{} {}", self.x, self.y)
    }

    fn y_diff(&self, o: &Point) -> i64 {
        (self.y as i64) - (o.y as i64)
    }
}

impl Add for Point {
    type Output = Point;

    fn add(self, other: Point) -> Point {
        Point {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl Sub for Point {
    type Output = Point;

    fn sub(self, other: Point) -> Point {
        Point {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

#[derive(Ord, PartialOrd, Eq, PartialEq)]
struct Edge {
    source: u32,
    target: u32,
}

struct Node {
    column: u8,
    name: String,
}

#[derive(Debug)]
struct Rect {
    height: u32,
    width: u32,
    top_left: Point,
}

// Ideally this would be a sequence doing something like 0..-8..8..-4..4..-6..6..-2..2
fn spacing_seq(widest: i32, stroke_width: i32) -> Vec<i32> {
    let mut result = vec![0];
    let mut curr = widest - (2 * stroke_width);
    let mut quarter_up = true;

    while curr.abs() != stroke_width {
        result.push(curr);
        if curr > 0 {
            curr = -curr;
        } else if quarter_up {
            curr = (curr / 2) + (-curr);
            quarter_up = false
        } else {
            curr = (-curr) - (curr / 2);
            quarter_up = true
        }
    }

    result
}

fn main() {
    let default_height = 40;
    let default_width = 120;
    let default_gap = 24;

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
                if start != end && start_col <= end_col && (*start_col as i32 - *end_col as i32).abs() <= 1 {
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

    let mut edges_in_lane: HashMap<u32, u32> = HashMap::new();

    let space_out_edges = |starting_at: u32, n: usize| {
        let seq: Vec<i32> = spacing_seq((default_gap / 2) as i32, 2);
        let index = n % (seq.len() - 1);

        ((starting_at as i32) + seq.get(index).unwrap())
    };

    let edges_str = edges
        .iter()
        .filter_map(|Edge { source, target }| {
            let start = placed_recs.get(*source as usize)?;
            let end = placed_recs.get(*target as usize)?;
            let lane_counts = edges_in_lane.clone();

            let mut lane_edge_count = lane_counts.get(&start.top_left.x);
            let same_level = start.top_left.x == end.top_left.x;
            let path_start = {
                if same_level {
                    *edges_in_lane.entry(start.top_left.x).or_insert(0) += 1;
                    start.top_left.clone()
                        + Point {
                            x: 0,
                            y: start.height / 2,
                        }
                } else {
                    *edges_in_lane.entry(end.top_left.x).or_insert(0) += 1;
                    lane_edge_count = edges_in_lane.get(&end.top_left.x);

                    start.top_left.clone()
                        + Point {
                            x: start.width,
                            y: start.height / 2,
                        }
                }
            };
            let horizontal_travel = {
                let gap =
                    space_out_edges(default_gap / 2, (*lane_edge_count.unwrap_or(&0)) as usize);

                if same_level {
                    gap * -1
                } else {
                    gap
                }
            };

            let vertical_travel = end.top_left.y_diff(&start.top_left);
            let path_end = (end.top_left.clone()
                + Point {
                    x: 0,
                    y: end.height / 2,
                })
            .svg_path_point();

            let next = format!(
                r#"<path d="
                    M {}
                    h {}
                    v {}
                    L {}" />"#,
                path_start.svg_path_point(),
                horizontal_travel,
                vertical_travel,
                path_end
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
          stroke-width: 1px;
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
        max_height,
        max_width + default_gap,
        edges_str,
        rect_str,
    );

    fs::write("index.html", svg).expect("Could not write svg out.")
}
