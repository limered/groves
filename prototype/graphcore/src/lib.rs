//! Throwaway prototype core: commit model, synthetic generator, git-log loader
//! and a straight-lane layout (pvigier-style, simplified). Shared by both UI
//! prototypes so they draw the exact same graph.

use std::collections::HashMap;
use std::time::Instant;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RefKind {
    Local,
    Remote,
    /// local branch and its upstream on the same commit
    Synced,
    Tag,
    Head,
}

#[derive(Clone, Debug)]
pub struct RefLabel {
    pub name: String,
    pub kind: RefKind,
}

#[derive(Clone, Debug)]
pub struct Commit {
    pub sha: String,
    pub parents: Vec<u32>,
    pub author: String,
    pub time: i64,
    pub subject: String,
    pub refs: Vec<RefLabel>,
}

#[derive(Clone, Copy, Debug)]
pub struct Edge {
    /// lane at row i
    pub src: u16,
    /// lane at row i+1
    pub dst: u16,
    pub color: u16,
}

pub struct Graph {
    pub commits: Vec<Commit>,
    /// lane of each row's dot
    pub lane: Vec<u16>,
    pub color: Vec<u16>,
    /// edges in the gap between row i and row i+1: edges[edge_off[i]..edge_off[i+1]]
    pub edges: Vec<Edge>,
    pub edge_off: Vec<u32>,
    pub max_lanes: u16,
    pub head_row: usize,
    pub layout_ms: f64,
    pub load_ms: f64,
}

impl Graph {
    pub fn gap_edges(&self, row: usize) -> &[Edge] {
        if row + 1 >= self.edge_off.len() {
            return &[];
        }
        &self.edges[self.edge_off[row] as usize..self.edge_off[row + 1] as usize]
    }

    pub fn build(commits: Vec<Commit>, load_ms: f64) -> Graph {
        let t = Instant::now();
        let n = commits.len();
        let mut lane = vec![0u16; n];
        let mut color = vec![0u16; n];
        let mut edges: Vec<Edge> = Vec::with_capacity(n * 4);
        let mut edge_off: Vec<u32> = Vec::with_capacity(n + 1);
        edge_off.push(0);

        let mut active: Vec<Option<u32>> = Vec::new();
        let mut lane_color: Vec<u16> = Vec::new();
        let mut next_color: u16 = 0;
        // pending gap edges from previous row: (src lane, slot)
        let mut pending: Vec<(u16, u16)> = Vec::new();
        let mut max_lanes = 0u16;

        let alloc = |active: &mut Vec<Option<u32>>, lane_color: &mut Vec<u16>, next_color: &mut u16, avoid: usize| -> usize {
            let slot = active
                .iter()
                .enumerate()
                .position(|(k, a)| a.is_none() && k != avoid)
                .unwrap_or_else(|| {
                    active.push(None);
                    lane_color.push(0);
                    active.len() - 1
                });
            lane_color[slot] = *next_color;
            *next_color = next_color.wrapping_add(1);
            slot
        };

        for i in 0..n {
            let me = i as u32;
            // find lanes expecting this commit
            let mut l_opt: Option<usize> = None;
            for (k, a) in active.iter().enumerate() {
                if *a == Some(me) {
                    l_opt = Some(k);
                    break;
                }
            }
            let l = match l_opt {
                Some(k) => k,
                None => alloc(&mut active, &mut lane_color, &mut next_color, usize::MAX),
            };
            // resolve pending edges of the previous gap
            if i > 0 {
                for &(src, slot) in &pending {
                    let dst = if active[slot as usize] == Some(me) { l as u16 } else { slot };
                    edges.push(Edge { src, dst, color: lane_color[slot as usize] });
                }
                edge_off.push(edges.len() as u32);
            }
            // converge other lanes expecting me
            for a in active.iter_mut() {
                if *a == Some(me) {
                    *a = None;
                }
            }
            lane[i] = l as u16;
            color[i] = lane_color[l];

            pending.clear();
            let parents = &commits[i].parents;
            let mut merge_slots: Vec<usize> = Vec::new();
            if let Some(&p0) = parents.first() {
                active[l] = Some(p0);
            }
            for &p in parents.iter().skip(1) {
                if let Some(k) = active.iter().position(|a| *a == Some(p)) {
                    pending.push((l as u16, k as u16));
                } else {
                    let m = alloc(&mut active, &mut lane_color, &mut next_color, l);
                    active[m] = Some(p);
                    merge_slots.push(m);
                    pending.push((l as u16, m as u16));
                }
            }
            for (k, a) in active.iter().enumerate() {
                if a.is_some() && !merge_slots.contains(&k) {
                    pending.push((k as u16, k as u16));
                }
            }
            // trim trailing free lanes
            while matches!(active.last(), Some(None)) {
                active.pop();
                lane_color.pop();
            }
            max_lanes = max_lanes.max(active.len() as u16).max(l as u16 + 1);
        }
        edge_off.push(edges.len() as u32);

        let head_row = commits
            .iter()
            .position(|c| c.refs.iter().any(|r| r.kind == RefKind::Head))
            .unwrap_or(0);

        Graph {
            commits,
            lane,
            color,
            edges,
            edge_off,
            max_lanes,
            head_row,
            layout_ms: t.elapsed().as_secs_f64() * 1000.0,
            load_ms,
        }
    }
}

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
    fn below(&mut self, n: u64) -> u64 {
        self.next() % n
    }
    fn chance(&mut self, pct: u64) -> bool {
        self.below(100) < pct
    }
}

const AUTHORS: &[&str] = &[
    "Emil Bohleber", "Anna Schmidt", "Jonas Weber", "Lea Fischer", "Max Wagner",
    "Sophie Becker", "Paul Hoffmann", "Mia Schulz", "dependabot[bot]", "Factory Bot",
];
const VERBS: &[&str] = &["Add", "Fix", "Refactor", "Update", "Remove", "Improve", "Rename", "Bump", "Merge"];
const THINGS: &[&str] = &[
    "graph layout for merge commits", "tenant settings page", "login redirect", "migration for invoices",
    "NuGet dependencies", "frontend lint warnings", "API client generation", "pipeline cache step",
    "German translations", "unit tests for scheduler", "osv-scanner ignore list", "workspace persistence",
];

/// Synthetic PR-style history: long main line, short feature branches merged back,
/// a few long-lived release branches, tags on main. Built oldest-first then reversed.
pub fn synthetic(n: usize, seed: u64) -> Graph {
    let t = Instant::now();
    let mut rng = Rng(seed | 1);
    // oldest-first: parents reference older indices
    struct Raw {
        parents: Vec<usize>,
        branch: usize,
    }
    let mut raw: Vec<Raw> = Vec::with_capacity(n);
    raw.push(Raw { parents: vec![], branch: 0 });
    let mut tips: Vec<Option<usize>> = vec![Some(0)]; // branch 0 = main
    let mut names: Vec<String> = vec!["main".into()];
    let mut open: Vec<usize> = vec![];
    let mut releases: Vec<usize> = vec![];
    let mut tag_rows: Vec<(usize, String)> = vec![];
    let mut main_count = 0usize;

    while raw.len() < n {
        let r = rng.below(100);
        let idx = raw.len();
        if r < 12 && open.len() < 14 {
            // start feature branch from main
            let b = tips.len();
            tips.push(tips[0]);
            names.push(format!("feature/A2M-{}-{}", 1000 + b, THINGS[rng.below(THINGS.len() as u64) as usize].split(' ').next().unwrap().to_lowercase()));
            open.push(b);
            continue;
        } else if r < 22 && !open.is_empty() {
            // merge a feature into main
            let k = rng.below(open.len() as u64) as usize;
            let b = open.swap_remove(k);
            raw.push(Raw { parents: vec![tips[0].unwrap(), tips[b].unwrap()], branch: 0 });
            tips[0] = Some(idx);
            tips[b] = None;
            main_count += 1;
        } else if r < 23 && releases.len() < 3 {
            let b = tips.len();
            tips.push(tips[0]);
            names.push(format!("release/{}.{}", 1 + main_count / 800, (main_count / 80) % 10));
            releases.push(b);
            continue;
        } else if r < 24 && !releases.is_empty() {
            let b = releases.remove(0);
            raw.push(Raw { parents: vec![tips[0].unwrap(), tips[b].unwrap()], branch: 0 });
            tips[0] = Some(idx);
            main_count += 1;
            // keep release tip alive for pills
            releases.push(b);
            if releases.len() > 3 {
                releases.remove(0);
            }
        } else if r < 40 || open.is_empty() {
            raw.push(Raw { parents: vec![tips[0].unwrap()], branch: 0 });
            tips[0] = Some(idx);
            main_count += 1;
            if main_count % 40 == 0 {
                tag_rows.push((idx, format!("v{}.{}.{}", 1 + main_count / 4000, (main_count / 400) % 10, (main_count / 40) % 10)));
            }
        } else {
            let pool: Vec<usize> = open.iter().chain(releases.iter()).copied().collect();
            let b = pool[rng.below(pool.len() as u64) as usize];
            raw.push(Raw { parents: vec![tips[b].unwrap()], branch: b });
            tips[b] = Some(idx);
        }
    }

    let total = raw.len();
    let rev = |i: usize| (total - 1 - i) as u32;
    let now = 1_790_000_000i64;
    let mut refs: Vec<Vec<RefLabel>> = vec![vec![]; total];
    for (b, tip) in tips.iter().enumerate() {
        if let Some(t) = tip {
            let kind = match b {
                0 => RefKind::Synced,
                _ if rng.chance(50) => RefKind::Synced,
                _ if rng.chance(50) => RefKind::Remote,
                _ => RefKind::Local,
            };
            let name = if kind == RefKind::Remote { format!("origin/{}", names[b]) } else { names[b].clone() };
            refs[*t].push(RefLabel { name, kind });
            if b == 0 {
                refs[*t].insert(0, RefLabel { name: "HEAD".into(), kind: RefKind::Head });
                refs[*t].push(RefLabel { name: "origin/HEAD".into(), kind: RefKind::Remote });
            }
        }
    }
    for (row, name) in tag_rows {
        refs[row].push(RefLabel { name, kind: RefKind::Tag });
    }

    let mut commits: Vec<Commit> = Vec::with_capacity(total);
    for i in (0..total).rev() {
        let r = &raw[i];
        let is_merge = r.parents.len() > 1;
        let subject = if is_merge {
            format!("Merged PR {}: {}", 20000 + i, THINGS[(i * 7) % THINGS.len()])
        } else {
            format!("{} {}", VERBS[(i * 31 + r.branch) % VERBS.len()], THINGS[(i * 13) % THINGS.len()])
        };
        commits.push(Commit {
            sha: format!("{:016x}{:024x}", (i as u64).wrapping_mul(0x9E3779B97F4A7C15), i),
            parents: r.parents.iter().map(|&p| rev(p)).collect(),
            author: AUTHORS[(i * 17 + r.branch * 3) % AUTHORS.len()].to_string(),
            time: now - ((total - i) as i64) * 1080,
            subject,
            refs: std::mem::take(&mut refs[i]),
        });
    }
    Graph::build(commits, t.elapsed().as_secs_f64() * 1000.0)
}

/// Load a real repository by shelling out to `git log` (prototype only; the real
/// thing uses gix). Loads all refs incl. tags.
pub fn from_git(path: &str) -> Result<Graph, String> {
    let t = Instant::now();
    let out = std::process::Command::new("git")
        .args(["-C", path, "log", "--all", "--topo-order", "--decorate=full", "--format=%H%x1f%P%x1f%an%x1f%at%x1f%D%x1f%s"])
        .output()
        .map_err(|e| e.to_string())?;
    if !out.status.success() {
        return Err(String::from_utf8_lossy(&out.stderr).into());
    }
    let text = String::from_utf8_lossy(&out.stdout);
    let mut lines: Vec<(String, Vec<String>, String, i64, String, String)> = Vec::new();
    for line in text.lines() {
        let f: Vec<&str> = line.splitn(6, '\x1f').collect();
        if f.len() < 6 {
            continue;
        }
        lines.push((
            f[0].into(),
            f[1].split_whitespace().map(String::from).collect(),
            f[2].into(),
            f[3].parse().unwrap_or(0),
            f[4].into(),
            f[5].into(),
        ));
    }
    let index: HashMap<&str, u32> = lines.iter().enumerate().map(|(i, l)| (l.0.as_str(), i as u32)).collect();
    let mut commits = Vec::with_capacity(lines.len());
    for l in &lines {
        let mut refs = Vec::new();
        let mut locals = Vec::new();
        let mut remotes = Vec::new();
        for d in l.4.split(", ").filter(|s| !s.is_empty()) {
            if let Some(b) = d.strip_prefix("HEAD -> refs/heads/") {
                refs.push(RefLabel { name: "HEAD".into(), kind: RefKind::Head });
                locals.push(b.to_string());
            } else if d == "HEAD" {
                refs.push(RefLabel { name: "HEAD".into(), kind: RefKind::Head });
            } else if let Some(b) = d.strip_prefix("refs/heads/") {
                locals.push(b.to_string());
            } else if let Some(b) = d.strip_prefix("refs/remotes/") {
                remotes.push(b.to_string());
            } else if let Some(b) = d.strip_prefix("tag: refs/tags/") {
                refs.push(RefLabel { name: b.into(), kind: RefKind::Tag });
            }
        }
        for b in locals {
            let up = format!("origin/{b}");
            if let Some(p) = remotes.iter().position(|r| *r == up) {
                remotes.remove(p);
                refs.push(RefLabel { name: b, kind: RefKind::Synced });
            } else {
                refs.push(RefLabel { name: b, kind: RefKind::Local });
            }
        }
        for r in remotes {
            refs.push(RefLabel { name: r, kind: RefKind::Remote });
        }
        commits.push(Commit {
            sha: l.0.clone(),
            parents: l.1.iter().filter_map(|p| index.get(p.as_str()).copied()).collect(),
            author: l.2.clone(),
            time: l.3,
            subject: l.5.clone(),
            refs,
        });
    }
    Ok(Graph::build(commits, t.elapsed().as_secs_f64() * 1000.0))
}

/// `--repo PATH` loads a real repo, `--n N` sets synthetic size (default 200k).
pub fn from_args() -> Graph {
    let args: Vec<String> = std::env::args().collect();
    let get = |k: &str| args.iter().position(|a| a == k).and_then(|i| args.get(i + 1)).cloned();
    if let Some(repo) = get("--repo") {
        match from_git(&repo) {
            Ok(g) => return g,
            Err(e) => eprintln!("git load failed: {e}; falling back to synthetic"),
        }
    }
    let n = get("--n").and_then(|s| s.parse().ok()).unwrap_or(200_000);
    synthetic(n, 0xC0FFEE)
}

pub const PALETTE: [(u8, u8, u8); 10] = [
    (0x4F, 0xC3, 0xF7), (0xFF, 0x8A, 0x65), (0x81, 0xC7, 0x84), (0xBA, 0x68, 0xC8), (0xFF, 0xD5, 0x4F),
    (0x4D, 0xB6, 0xAC), (0xF0, 0x62, 0x92), (0x90, 0xA4, 0xAE), (0xAE, 0xD5, 0x81), (0x79, 0x86, 0xCB),
];

pub fn rel_time(t: i64, now: i64) -> String {
    let d = (now - t).max(0);
    match d {
        0..=3599 => format!("{}m", d / 60),
        3600..=86_399 => format!("{}h", d / 3600),
        86_400..=2_591_999 => format!("{}d", d / 86_400),
        _ => format!("{}mo", d / 2_592_000),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn layout_200k() {
        let g = synthetic(200_000, 42);
        assert_eq!(g.commits.len(), 200_000);
        assert!(g.max_lanes < 40, "lanes {}", g.max_lanes);
        eprintln!("gen {:.1} ms, layout {:.1} ms, lanes {}, edges {}", g.load_ms, g.layout_ms, g.max_lanes, g.edges.len());
    }
}
