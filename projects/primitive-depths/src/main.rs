//! # Primitive Depths
//!
//! A terminal roguelike where you descend through procedurally generated
//! dungeons collecting all 16 Lex Primitiva symbols. Each primitive grants
//! a unique power. Defeat antipattern enemies. Master the depths.

use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal::{self, ClearType},
};
use rand::Rng;
use std::io::{self, Write};
use std::time::Duration;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

const MAP_W: usize = 70;
const MAP_H: usize = 20;
const MAX_ROOMS: usize = 9;
const MIN_ROOM_W: usize = 5;
const MAX_ROOM_W: usize = 12;
const MIN_ROOM_H: usize = 3;
const MAX_ROOM_H: usize = 7;
const NUM_FLOORS: usize = 5;
const PRIMS_PER_FLOOR: usize = 4;
const VIEW_RADIUS: i32 = 8;

// ---------------------------------------------------------------------------
// Lex Primitiva definitions
// ---------------------------------------------------------------------------

struct PrimDef {
    symbol: &'static str,
    name: &'static str,
    desc: &'static str,
    color: Color,
}

const PRIMITIVES: [PrimDef; 16] = [
    PrimDef { symbol: "S", name: "Sequence",       desc: "+2 ATK (ordered strikes)",    color: Color::Cyan },
    PrimDef { symbol: "M", name: "Mapping",        desc: "+2 DEF (transform damage)",   color: Color::Blue },
    PrimDef { symbol: "s", name: "State",           desc: "+8 Max HP (mutable vitality)", color: Color::Green },
    PrimDef { symbol: "r", name: "Recursion",       desc: "Full heal! (self-reference)", color: Color::Magenta },
    PrimDef { symbol: "0", name: "Void",            desc: "Enemies -2 ATK (nullify)",    color: Color::DarkGrey },
    PrimDef { symbol: "B", name: "Boundary",        desc: "+3 DEF (edge protection)",    color: Color::Yellow },
    PrimDef { symbol: "f", name: "Frequency",       desc: "+1 ATK, +1 DEF (tempo)",      color: Color::DarkCyan },
    PrimDef { symbol: "E", name: "Existence",       desc: "Reveal floor (detection)",    color: Color::White },
    PrimDef { symbol: "P", name: "Persistence",     desc: "+12 Max HP (endurance)",      color: Color::DarkGreen },
    PrimDef { symbol: ">", name: "Causality",       desc: "+4 ATK (cause & effect)",     color: Color::Red },
    PrimDef { symbol: "K", name: "Comparison",      desc: "+2 ATK, +1 DEF (judgment)",   color: Color::DarkYellow },
    PrimDef { symbol: "N", name: "Quantity",         desc: "+20 HP heal (abundance)",     color: Color::Cyan },
    PrimDef { symbol: "L", name: "Location",         desc: "See enemies (awareness)",     color: Color::DarkMagenta },
    PrimDef { symbol: "I", name: "Irreversibility",  desc: "+3 ATK (permanent force)",    color: Color::DarkRed },
    PrimDef { symbol: "+", name: "Sum",              desc: "+5 HP, +2 ATK (combine)",     color: Color::Green },
    PrimDef { symbol: "X", name: "Product",          desc: "+2 ATK, +2 DEF (multiply)",   color: Color::Yellow },
];

// ---------------------------------------------------------------------------
// Tiles
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq)]
enum Tile {
    Wall,
    Floor,
    StairsDown,
}

// ---------------------------------------------------------------------------
// Room (for dungeon generation)
// ---------------------------------------------------------------------------

struct Room {
    x: usize,
    y: usize,
    w: usize,
    h: usize,
}

impl Room {
    fn center(&self) -> (usize, usize) {
        (self.x + self.w / 2, self.y + self.h / 2)
    }

    fn intersects(&self, other: &Room) -> bool {
        self.x < other.x + other.w + 2
            && self.x + self.w + 2 > other.x
            && self.y < other.y + other.h + 2
            && self.y + self.h + 2 > other.y
    }
}

// ---------------------------------------------------------------------------
// Enemy
// ---------------------------------------------------------------------------

struct Enemy {
    x: usize,
    y: usize,
    glyph: char,
    name: String,
    hp: i32,
    max_hp: i32,
    atk: i32,
    def: i32,
    color: Color,
    xp_value: u32,
}

// ---------------------------------------------------------------------------
// Primitive on the map
// ---------------------------------------------------------------------------

struct PrimOnMap {
    x: usize,
    y: usize,
    idx: usize,
}

// ---------------------------------------------------------------------------
// Player
// ---------------------------------------------------------------------------

struct Player {
    x: usize,
    y: usize,
    hp: i32,
    max_hp: i32,
    atk: i32,
    def: i32,
    collected: Vec<usize>,
    xp: u32,
    level: u32,
    seen: Vec<Vec<bool>>,
    see_enemies: bool,
    revealed: bool,
}

impl Player {
    fn new() -> Self {
        Self {
            x: 0,
            y: 0,
            hp: 30,
            max_hp: 30,
            atk: 5,
            def: 2,
            collected: Vec::new(),
            xp: 0,
            level: 1,
            seen: vec![vec![false; MAP_W]; MAP_H],
            see_enemies: false,
            revealed: false,
        }
    }

    fn reset_vision(&mut self) {
        self.seen = vec![vec![false; MAP_W]; MAP_H];
        self.revealed = false;
    }
}

// ---------------------------------------------------------------------------
// Game
// ---------------------------------------------------------------------------

struct Game {
    map: Vec<Vec<Tile>>,
    player: Player,
    enemies: Vec<Enemy>,
    prims: Vec<PrimOnMap>,
    log: Vec<(String, Color)>,
    floor: usize,
    turn: u32,
    game_over: bool,
    victory: bool,
    remaining: Vec<usize>,
}

impl Game {
    fn new() -> Self {
        let remaining: Vec<usize> = (0..16).collect();
        let mut g = Game {
            map: vec![vec![Tile::Wall; MAP_W]; MAP_H],
            player: Player::new(),
            enemies: Vec::new(),
            prims: Vec::new(),
            log: vec![("Welcome to Primitive Depths!".into(), Color::Cyan)],
            floor: 1,
            turn: 0,
            game_over: false,
            victory: false,
            remaining,
        };
        g.generate_floor();
        g.msg("Collect all 16 Lex Primitiva. Descend the stairs.", Color::White);
        g.msg("[Arrow/WASD] move  [.] wait  [q] quit", Color::DarkGrey);
        g
    }

    fn msg(&mut self, text: &str, color: Color) {
        self.log.push((text.to_string(), color));
    }

    // -----------------------------------------------------------------------
    // Dungeon generation
    // -----------------------------------------------------------------------

    fn generate_floor(&mut self) {
        let mut rng = rand::thread_rng();
        self.map = vec![vec![Tile::Wall; MAP_W]; MAP_H];
        self.enemies.clear();
        self.prims.clear();
        self.player.reset_vision();

        // Generate rooms
        let mut rooms: Vec<Room> = Vec::new();
        for _ in 0..50 {
            if rooms.len() >= MAX_ROOMS {
                break;
            }
            let w = rng.gen_range(MIN_ROOM_W..=MAX_ROOM_W);
            let h = rng.gen_range(MIN_ROOM_H..=MAX_ROOM_H);
            let x = rng.gen_range(1..MAP_W.saturating_sub(w + 1).max(2));
            let y = rng.gen_range(1..MAP_H.saturating_sub(h + 1).max(2));
            let room = Room { x, y, w, h };
            if rooms.iter().all(|r| !r.intersects(&room)) {
                rooms.push(room);
            }
        }

        // Ensure at least 3 rooms
        if rooms.len() < 3 {
            rooms.clear();
            rooms.push(Room { x: 2, y: 2, w: 8, h: 5 });
            rooms.push(Room { x: 30, y: 2, w: 8, h: 5 });
            rooms.push(Room { x: 15, y: 12, w: 10, h: 5 });
        }

        // Carve rooms
        for room in &rooms {
            for dy in 0..room.h {
                for dx in 0..room.w {
                    let ry = room.y + dy;
                    let rx = room.x + dx;
                    if ry < MAP_H && rx < MAP_W {
                        self.map[ry][rx] = Tile::Floor;
                    }
                }
            }
        }

        // Connect rooms with corridors
        for i in 1..rooms.len() {
            let (cx1, cy1) = rooms[i - 1].center();
            let (cx2, cy2) = rooms[i].center();
            if rng.gen_bool(0.5) {
                self.carve_h(cx1, cx2, cy1);
                self.carve_v(cy1, cy2, cx2);
            } else {
                self.carve_v(cy1, cy2, cx1);
                self.carve_h(cx1, cx2, cy2);
            }
        }

        // Player in first room
        let (px, py) = rooms[0].center();
        self.player.x = px;
        self.player.y = py;

        // Stairs in last room
        let (sx, sy) = rooms[rooms.len() - 1].center();
        self.map[sy][sx] = Tile::StairsDown;

        // Place primitives
        let to_place = PRIMS_PER_FLOOR.min(self.remaining.len());
        let mut placed = 0;
        for _ in 0..200 {
            if placed >= to_place {
                break;
            }
            let ri = rng.gen_range(1..rooms.len());
            let room = &rooms[ri];
            let fx = room.x + rng.gen_range(0..room.w);
            let fy = room.y + rng.gen_range(0..room.h);
            if fx < MAP_W
                && fy < MAP_H
                && self.map[fy][fx] == Tile::Floor
                && !(fx == px && fy == py)
                && !(fx == sx && fy == sy)
                && !self.prims.iter().any(|p| p.x == fx && p.y == fy)
            {
                self.prims.push(PrimOnMap {
                    x: fx,
                    y: fy,
                    idx: self.remaining[placed],
                });
                placed += 1;
            }
        }

        // Place enemies
        let num_enemies = self.floor + 2;
        self.spawn_enemies(&rooms, num_enemies, &mut rng, px, py);

        self.msg(
            &format!("--- Floor {}/{} ---", self.floor, NUM_FLOORS),
            Color::Yellow,
        );

        // Update vision
        self.update_fov();
    }

    fn spawn_enemies(
        &mut self,
        rooms: &[Room],
        count: usize,
        rng: &mut impl Rng,
        px: usize,
        py: usize,
    ) {
        let templates: Vec<(char, &str, i32, i32, i32, Color, u32)> = match self.floor {
            1 => vec![
                ('g', "Unwrap Goblin", 8, 3, 0, Color::Green, 10),
                ('r', "Raw Ptr Rat", 5, 4, 0, Color::DarkGreen, 8),
            ],
            2 => vec![
                ('g', "Unwrap Goblin", 10, 4, 1, Color::Green, 12),
                ('p', "Panic Sprite", 7, 6, 0, Color::Red, 15),
                ('r', "Raw Ptr Rat", 6, 5, 0, Color::DarkGreen, 10),
            ],
            3 => vec![
                ('o', "Unsafe Orc", 16, 5, 2, Color::Yellow, 20),
                ('p', "Panic Sprite", 9, 7, 0, Color::Red, 18),
                ('w', "Deadlock Wraith", 12, 6, 1, Color::DarkCyan, 22),
            ],
            4 => vec![
                ('o', "Unsafe Orc", 20, 6, 3, Color::Yellow, 25),
                ('T', "Clone Troll", 24, 5, 4, Color::Magenta, 30),
                ('w', "Deadlock Wraith", 15, 7, 2, Color::DarkCyan, 28),
            ],
            _ => vec![
                ('D', "Dragon Leak", 35, 9, 5, Color::DarkRed, 50),
                ('T', "Clone Troll", 28, 7, 4, Color::Magenta, 35),
                ('w', "Deadlock Wraith", 20, 8, 3, Color::DarkCyan, 32),
            ],
        };

        let mut spawned = 0;
        for _ in 0..300 {
            if spawned >= count {
                break;
            }
            if rooms.len() < 2 {
                break;
            }
            let ri = rng.gen_range(1..rooms.len());
            let room = &rooms[ri];
            let ex = room.x + rng.gen_range(0..room.w);
            let ey = room.y + rng.gen_range(0..room.h);
            if ex < MAP_W
                && ey < MAP_H
                && self.map[ey][ex] == Tile::Floor
                && !(ex == px && ey == py)
                && !self.enemies.iter().any(|e| e.x == ex && e.y == ey)
            {
                let t = &templates[rng.gen_range(0..templates.len())];
                self.enemies.push(Enemy {
                    x: ex,
                    y: ey,
                    glyph: t.0,
                    name: t.1.to_string(),
                    hp: t.2,
                    max_hp: t.2,
                    atk: t.3,
                    def: t.4,
                    color: t.5,
                    xp_value: t.6,
                });
                spawned += 1;
            }
        }
    }

    fn carve_h(&mut self, x1: usize, x2: usize, y: usize) {
        let (a, b) = if x1 < x2 { (x1, x2) } else { (x2, x1) };
        for x in a..=b {
            if y < MAP_H && x < MAP_W {
                self.map[y][x] = Tile::Floor;
            }
        }
    }

    fn carve_v(&mut self, y1: usize, y2: usize, x: usize) {
        let (a, b) = if y1 < y2 { (y1, y2) } else { (y2, y1) };
        for y in a..=b {
            if y < MAP_H && x < MAP_W {
                self.map[y][x] = Tile::Floor;
            }
        }
    }

    // -----------------------------------------------------------------------
    // Field of view (simple raycasting approximation)
    // -----------------------------------------------------------------------

    fn update_fov(&mut self) {
        if self.player.revealed {
            for row in &mut self.player.seen {
                for cell in row.iter_mut() {
                    *cell = true;
                }
            }
            return;
        }

        let px = self.player.x as i32;
        let py = self.player.y as i32;

        // Simple shadow-free radius check
        for dy in -VIEW_RADIUS..=VIEW_RADIUS {
            for dx in -VIEW_RADIUS..=VIEW_RADIUS {
                if dx * dx + dy * dy > VIEW_RADIUS * VIEW_RADIUS {
                    continue;
                }
                let tx = px + dx;
                let ty = py + dy;
                if tx >= 0 && ty >= 0 && (tx as usize) < MAP_W && (ty as usize) < MAP_H {
                    // Simple line-of-sight check
                    if self.has_los(px, py, tx, ty) {
                        self.player.seen[ty as usize][tx as usize] = true;
                    }
                }
            }
        }
    }

    fn has_los(&self, x0: i32, y0: i32, x1: i32, y1: i32) -> bool {
        let dx = (x1 - x0).abs();
        let dy = -(y1 - y0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;
        let mut cx = x0;
        let mut cy = y0;

        loop {
            if cx == x1 && cy == y1 {
                return true;
            }
            if cx != x0 || cy != y0 {
                let ux = cx as usize;
                let uy = cy as usize;
                if ux < MAP_W && uy < MAP_H && self.map[uy][ux] == Tile::Wall {
                    return false;
                }
            }
            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                cx += sx;
            }
            if e2 <= dx {
                err += dx;
                cy += sy;
            }
        }
    }

    // -----------------------------------------------------------------------
    // Player actions
    // -----------------------------------------------------------------------

    fn try_move(&mut self, dx: i32, dy: i32) {
        let nx = self.player.x as i32 + dx;
        let ny = self.player.y as i32 + dy;

        if nx < 0 || ny < 0 {
            return;
        }
        let nx = nx as usize;
        let ny = ny as usize;
        if nx >= MAP_W || ny >= MAP_H {
            return;
        }
        if self.map[ny][nx] == Tile::Wall {
            return;
        }

        // Bump-attack enemy
        if let Some(idx) = self.enemies.iter().position(|e| e.x == nx && e.y == ny) {
            self.attack_enemy(idx);
            self.turn += 1;
            self.enemy_turns();
            self.update_fov();
            return;
        }

        self.player.x = nx;
        self.player.y = ny;
        self.turn += 1;

        // Pickup primitive
        self.check_pickup();

        // Check stairs
        if self.map[ny][nx] == Tile::StairsDown {
            self.descend();
        }

        self.enemy_turns();
        self.update_fov();
    }

    fn wait_turn(&mut self) {
        self.turn += 1;
        // Regen 1 HP when waiting
        if self.player.hp < self.player.max_hp {
            self.player.hp += 1;
        }
        self.enemy_turns();
        self.update_fov();
    }

    fn check_pickup(&mut self) {
        let px = self.player.x;
        let py = self.player.y;
        if let Some(pos) = self.prims.iter().position(|p| p.x == px && p.y == py) {
            let prim = self.prims.remove(pos);
            let def = &PRIMITIVES[prim.idx];
            self.player.collected.push(prim.idx);
            self.remaining.retain(|&i| i != prim.idx);

            // Apply bonus
            match prim.idx {
                0 => {
                    self.player.atk += 2;
                    self.msg(&format!("{} {}: {}", def.symbol, def.name, def.desc), def.color);
                }
                1 => {
                    self.player.def += 2;
                    self.msg(&format!("{} {}: {}", def.symbol, def.name, def.desc), def.color);
                }
                2 => {
                    self.player.max_hp += 8;
                    self.player.hp += 8;
                    self.msg(&format!("{} {}: {}", def.symbol, def.name, def.desc), def.color);
                }
                3 => {
                    self.player.hp = self.player.max_hp;
                    self.msg(&format!("{} {}: {}", def.symbol, def.name, def.desc), def.color);
                }
                4 => {
                    for e in &mut self.enemies {
                        e.atk = (e.atk - 2).max(1);
                    }
                    self.msg(&format!("{} {}: {}", def.symbol, def.name, def.desc), def.color);
                }
                5 => {
                    self.player.def += 3;
                    self.msg(&format!("{} {}: {}", def.symbol, def.name, def.desc), def.color);
                }
                6 => {
                    self.player.atk += 1;
                    self.player.def += 1;
                    self.msg(&format!("{} {}: {}", def.symbol, def.name, def.desc), def.color);
                }
                7 => {
                    self.player.revealed = true;
                    for row in &mut self.player.seen {
                        for cell in row.iter_mut() {
                            *cell = true;
                        }
                    }
                    self.msg(&format!("{} {}: {}", def.symbol, def.name, def.desc), def.color);
                }
                8 => {
                    self.player.max_hp += 12;
                    self.player.hp += 12;
                    self.msg(&format!("{} {}: {}", def.symbol, def.name, def.desc), def.color);
                }
                9 => {
                    self.player.atk += 4;
                    self.msg(&format!("{} {}: {}", def.symbol, def.name, def.desc), def.color);
                }
                10 => {
                    self.player.atk += 2;
                    self.player.def += 1;
                    self.msg(&format!("{} {}: {}", def.symbol, def.name, def.desc), def.color);
                }
                11 => {
                    self.player.hp = (self.player.hp + 20).min(self.player.max_hp);
                    self.msg(&format!("{} {}: {}", def.symbol, def.name, def.desc), def.color);
                }
                12 => {
                    self.player.see_enemies = true;
                    self.msg(&format!("{} {}: {}", def.symbol, def.name, def.desc), def.color);
                }
                13 => {
                    self.player.atk += 3;
                    self.msg(&format!("{} {}: {}", def.symbol, def.name, def.desc), def.color);
                }
                14 => {
                    self.player.max_hp += 5;
                    self.player.hp += 5;
                    self.player.atk += 2;
                    self.msg(&format!("{} {}: {}", def.symbol, def.name, def.desc), def.color);
                }
                15 => {
                    self.player.atk += 2;
                    self.player.def += 2;
                    self.msg(&format!("{} {}: {}", def.symbol, def.name, def.desc), def.color);
                }
                _ => {}
            }

            if self.player.collected.len() == 16 {
                self.msg(
                    "*** ALL 16 LEX PRIMITIVA COLLECTED! ***",
                    Color::Magenta,
                );
            }
        }
    }

    fn descend(&mut self) {
        if self.floor >= NUM_FLOORS {
            if self.player.collected.len() == 16 {
                self.victory = true;
                self.game_over = true;
                self.msg("VICTORY! You've mastered all Lex Primitiva!", Color::Green);
            } else {
                self.msg(
                    &format!(
                        "Need all 16 to ascend! ({}/16)",
                        self.player.collected.len()
                    ),
                    Color::Yellow,
                );
            }
        } else {
            self.floor += 1;
            self.generate_floor();
        }
    }

    // -----------------------------------------------------------------------
    // Combat
    // -----------------------------------------------------------------------

    fn attack_enemy(&mut self, idx: usize) {
        let mut rng = rand::thread_rng();
        let damage = (self.player.atk - self.enemies[idx].def + rng.gen_range(0..3)).max(1);
        self.enemies[idx].hp -= damage;
        let name = self.enemies[idx].name.clone();

        if self.enemies[idx].hp <= 0 {
            let xp = self.enemies[idx].xp_value;
            self.player.xp += xp;
            self.msg(
                &format!("{} slain! +{} XP  ({} dmg)", name, xp, damage),
                Color::Green,
            );
            self.enemies.remove(idx);
            self.check_level_up();
        } else {
            self.msg(
                &format!(
                    "Hit {} for {} dmg ({}/{})",
                    name, damage, self.enemies[idx].hp, self.enemies[idx].max_hp
                ),
                Color::White,
            );
        }
    }

    fn check_level_up(&mut self) {
        let threshold = self.player.level * 40;
        if self.player.xp >= threshold {
            self.player.level += 1;
            self.player.max_hp += 5;
            self.player.hp = self.player.max_hp;
            self.player.atk += 1;
            self.player.def += 1;
            self.msg(
                &format!(
                    "** LEVEL UP! -> Lv.{} ** (Full heal, +1 ATK/DEF)",
                    self.player.level
                ),
                Color::Magenta,
            );
        }
    }

    fn enemy_turns(&mut self) {
        let px = self.player.x;
        let py = self.player.y;
        let mut rng = rand::thread_rng();

        let n = self.enemies.len();
        for i in 0..n {
            if i >= self.enemies.len() {
                break;
            }
            let ex = self.enemies[i].x;
            let ey = self.enemies[i].y;
            let dx_abs = (px as i32 - ex as i32).abs();
            let dy_abs = (py as i32 - ey as i32).abs();
            let dist = dx_abs + dy_abs;

            if dist <= 1 {
                // Attack player
                let damage =
                    (self.enemies[i].atk - self.player.def + rng.gen_range(0..2)).max(1);
                self.player.hp -= damage;
                let name = self.enemies[i].name.clone();
                self.msg(&format!("{} hits you for {}!", name, damage), Color::Red);

                if self.player.hp <= 0 {
                    self.game_over = true;
                    self.msg("You have been defeated...", Color::DarkRed);
                    return;
                }
            } else if dist < 10 {
                // Chase player
                let ddx: i32 = if px > ex {
                    1
                } else if px < ex {
                    -1
                } else {
                    0
                };
                let ddy: i32 = if py > ey {
                    1
                } else if py < ey {
                    -1
                } else {
                    0
                };

                // Try primary direction, then secondary
                let moves = if rng.gen_bool(0.6) {
                    [(ddx, 0), (0, ddy)]
                } else {
                    [(0, ddy), (ddx, 0)]
                };

                let mut moved = false;
                for (mdx, mdy) in moves {
                    if mdx == 0 && mdy == 0 {
                        continue;
                    }
                    let nx = (ex as i32 + mdx) as usize;
                    let ny = (ey as i32 + mdy) as usize;
                    if nx < MAP_W
                        && ny < MAP_H
                        && self.map[ny][nx] != Tile::Wall
                        && !(nx == px && ny == py)
                        && !self.enemies.iter().enumerate().any(|(j, e)| {
                            j != i && e.x == nx && e.y == ny
                        })
                    {
                        self.enemies[i].x = nx;
                        self.enemies[i].y = ny;
                        moved = true;
                        break;
                    }
                }

                // Random wander if stuck
                if !moved && rng.gen_bool(0.3) {
                    let dirs = [(0i32, 1i32), (0, -1), (1, 0), (-1, 0)];
                    let d = dirs[rng.gen_range(0..4)];
                    let nx = (ex as i32 + d.0) as usize;
                    let ny = (ey as i32 + d.1) as usize;
                    if nx < MAP_W
                        && ny < MAP_H
                        && self.map[ny][nx] == Tile::Floor
                        && !(nx == px && ny == py)
                        && !self.enemies.iter().enumerate().any(|(j, e)| {
                            j != i && e.x == nx && e.y == ny
                        })
                    {
                        self.enemies[i].x = nx;
                        self.enemies[i].y = ny;
                    }
                }
            }
        }
    }

    // -----------------------------------------------------------------------
    // Rendering
    // -----------------------------------------------------------------------

    fn render(&self, out: &mut io::Stdout) -> io::Result<()> {
        execute!(out, cursor::MoveTo(0, 0))?;

        // Title bar
        execute!(
            out,
            SetForegroundColor(Color::Cyan),
            Print(format!(
                " PRIMITIVE DEPTHS   Floor {}/{}   Turn {}   Lv.{}\n",
                self.floor, NUM_FLOORS, self.turn, self.player.level
            )),
            ResetColor
        )?;

        // HP bar
        let hp_frac = if self.player.max_hp > 0 {
            self.player.hp as f32 / self.player.max_hp as f32
        } else {
            0.0
        };
        let bar_len = 25;
        let filled = (hp_frac * bar_len as f32) as usize;
        let hp_color = if hp_frac > 0.6 {
            Color::Green
        } else if hp_frac > 0.3 {
            Color::Yellow
        } else {
            Color::Red
        };

        execute!(out, Print(" HP "))?;
        for i in 0..bar_len {
            if i < filled {
                execute!(
                    out,
                    SetBackgroundColor(hp_color),
                    SetForegroundColor(Color::Black),
                    Print(" "),
                    ResetColor
                )?;
            } else {
                execute!(
                    out,
                    SetBackgroundColor(Color::DarkGrey),
                    Print(" "),
                    ResetColor
                )?;
            }
        }
        execute!(
            out,
            Print(format!(
                " {}/{}  ATK:{}  DEF:{}  XP:{}\n",
                self.player.hp.max(0),
                self.player.max_hp,
                self.player.atk,
                self.player.def,
                self.player.xp
            ))
        )?;

        // Map
        for y in 0..MAP_H {
            execute!(out, Print(" "))?;
            for x in 0..MAP_W {
                let visible = self.player.seen[y][x];
                let in_fov = self.in_fov(x as i32, y as i32);

                if x == self.player.x && y == self.player.y {
                    execute!(
                        out,
                        SetForegroundColor(Color::White),
                        SetBackgroundColor(Color::DarkBlue),
                        Print("@"),
                        ResetColor
                    )?;
                } else if let Some(e) = self.enemies.iter().find(|e| e.x == x && e.y == y)
                {
                    if in_fov || self.player.see_enemies {
                        execute!(
                            out,
                            SetForegroundColor(e.color),
                            Print(format!("{}", e.glyph)),
                            ResetColor
                        )?;
                    } else if visible {
                        execute!(
                            out,
                            SetForegroundColor(Color::DarkGrey),
                            Print("."),
                            ResetColor
                        )?;
                    } else {
                        execute!(out, Print(" "))?;
                    }
                } else if let Some(p) = self.prims.iter().find(|p| p.x == x && p.y == y)
                {
                    if in_fov || self.player.revealed {
                        let def = &PRIMITIVES[p.idx];
                        execute!(
                            out,
                            SetForegroundColor(Color::Black),
                            SetBackgroundColor(def.color),
                            Print(def.symbol),
                            ResetColor
                        )?;
                    } else if visible {
                        execute!(
                            out,
                            SetForegroundColor(Color::DarkGrey),
                            Print("."),
                            ResetColor
                        )?;
                    } else {
                        execute!(out, Print(" "))?;
                    }
                } else if in_fov {
                    match self.map[y][x] {
                        Tile::Wall => {
                            execute!(
                                out,
                                SetForegroundColor(Color::Grey),
                                Print("#"),
                                ResetColor
                            )?;
                        }
                        Tile::Floor => {
                            execute!(
                                out,
                                SetForegroundColor(Color::DarkGrey),
                                Print("."),
                                ResetColor
                            )?;
                        }
                        Tile::StairsDown => {
                            execute!(
                                out,
                                SetForegroundColor(Color::Yellow),
                                SetBackgroundColor(Color::DarkGrey),
                                Print(">"),
                                ResetColor
                            )?;
                        }
                    }
                } else if visible {
                    match self.map[y][x] {
                        Tile::Wall => {
                            execute!(
                                out,
                                SetForegroundColor(Color::DarkGrey),
                                Print("#"),
                                ResetColor
                            )?;
                        }
                        Tile::Floor | Tile::StairsDown => {
                            execute!(
                                out,
                                SetForegroundColor(Color::DarkGrey),
                                Print("."),
                                ResetColor
                            )?;
                        }
                    }
                } else {
                    execute!(out, Print(" "))?;
                }
            }
            execute!(out, Print("\n"))?;
        }

        // Primitive collection display
        execute!(out, Print(" Primitives: "))?;
        for i in 0..16 {
            let def = &PRIMITIVES[i];
            if self.player.collected.contains(&i) {
                execute!(
                    out,
                    SetForegroundColor(Color::Black),
                    SetBackgroundColor(def.color),
                    Print(def.symbol),
                    ResetColor,
                    Print(" ")
                )?;
            } else {
                execute!(
                    out,
                    SetForegroundColor(Color::DarkGrey),
                    Print(". "),
                    ResetColor
                )?;
            }
        }
        execute!(
            out,
            Print(format!(" ({}/16)\n", self.player.collected.len()))
        )?;

        // Message log (last 4)
        let start = if self.log.len() > 4 {
            self.log.len() - 4
        } else {
            0
        };
        for (text, color) in &self.log[start..] {
            execute!(
                out,
                Print(" "),
                SetForegroundColor(*color),
                Print(text),
                ResetColor,
                Print("\n")
            )?;
        }

        // Pad to fixed height
        let used_lines = 2 + MAP_H + 1 + 4; // header + hp + map + prims + 4 log
        let total_target = MAP_H + 10;
        for _ in used_lines..total_target {
            execute!(out, Print("\n"))?;
        }

        out.flush()?;
        Ok(())
    }

    fn in_fov(&self, x: i32, y: i32) -> bool {
        let px = self.player.x as i32;
        let py = self.player.y as i32;
        let dx = x - px;
        let dy = y - py;
        dx * dx + dy * dy <= VIEW_RADIUS * VIEW_RADIUS
            && self.has_los(px, py, x, y)
    }
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

fn main() -> io::Result<()> {
    let mut stdout = io::stdout();
    terminal::enable_raw_mode()?;
    execute!(
        stdout,
        terminal::EnterAlternateScreen,
        cursor::Hide,
        terminal::Clear(ClearType::All)
    )?;

    let mut game = Game::new();

    loop {
        execute!(stdout, terminal::Clear(ClearType::All))?;
        game.render(&mut stdout)?;

        if game.game_over {
            if game.victory {
                execute!(
                    stdout,
                    SetForegroundColor(Color::Green),
                    Print("\n *** VICTORY! All 16 Lex Primitiva mastered! ***\n"),
                    Print(" You have achieved T3 Domain Mastery.\n"),
                    ResetColor,
                    Print("\n Press any key to exit...\n")
                )?;
            } else {
                execute!(
                    stdout,
                    SetForegroundColor(Color::Red),
                    Print("\n *** GAME OVER ***\n"),
                    Print(" The antipatterns consumed you...\n"),
                    ResetColor,
                    Print(format!(
                        "\n Primitives collected: {}/16  Floor: {}  Turns: {}\n",
                        game.player.collected.len(),
                        game.floor,
                        game.turn
                    )),
                    Print(" Press any key to exit...\n")
                )?;
            }
            stdout.flush()?;
            loop {
                if let Event::Key(_) = event::read()? {
                    break;
                }
            }
            break;
        }

        if event::poll(Duration::from_millis(200))? {
            if let Event::Key(KeyEvent {
                code, modifiers, ..
            }) = event::read()?
            {
                match code {
                    KeyCode::Up | KeyCode::Char('k') => game.try_move(0, -1),
                    KeyCode::Down | KeyCode::Char('j') => game.try_move(0, 1),
                    KeyCode::Left | KeyCode::Char('h') => game.try_move(-1, 0),
                    KeyCode::Right | KeyCode::Char('l') => game.try_move(1, 0),
                    KeyCode::Char('y') => game.try_move(-1, -1),
                    KeyCode::Char('u') => game.try_move(1, -1),
                    KeyCode::Char('b') => game.try_move(-1, 1),
                    KeyCode::Char('n') => game.try_move(1, 1),
                    KeyCode::Char('.') | KeyCode::Char('5') => game.wait_turn(),
                    KeyCode::Char('q') => break,
                    KeyCode::Char('c') if modifiers.contains(KeyModifiers::CONTROL) => {
                        break;
                    }
                    _ => {}
                }
            }
        }
    }

    execute!(
        stdout,
        cursor::Show,
        terminal::LeaveAlternateScreen
    )?;
    terminal::disable_raw_mode()?;
    Ok(())
}
