//! plato-room-engine — Room Runtime Engine
//!
//! Rooms are applications. Applications are rooms.
//! This engine provides the runtime: create rooms, navigate between them,
//! execute room logic, and manage room lifecycle.
//!
//! ## Invisible Plumbing
//! - Room addressing via plato-address
//! - Event hooks via plato-hooks
//! - Tile storage via plato-tile-spec
//! - Scoring via plato-tile-scorer
//!
//! ## API
//! ```rust
//! let engine = RoomEngine::new("/data/rooms");
//! engine.create_room("math", RoomType::Application);
//! engine.navigate("math");
//! engine.execute("query: pythagorean theorem");
//! engine.navigate_back();
//! ```

use std::collections::HashMap;

/// Room types — a room can be many things.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoomType {
    /// A PLATO learning room with tiles and anchors.
    Learning,
    /// An application room (tool, service, agent workspace).
    Application,
    /// A social room (fleet communication, collaboration).
    Social,
    /// A system room (monitoring, config, fleet ops).
    System,
}

impl Default for RoomType {
    fn default() -> Self { RoomType::Learning }
}

/// Room metadata.
#[derive(Debug, Clone)]
pub struct Room {
    pub id: String,
    pub room_type: RoomType,
    pub description: String,
    pub created_at: u64,
    pub tile_count: usize,
    pub parent: Option<String>,
    pub children: Vec<String>,
    pub active: bool,
    pub metadata: HashMap<String, String>,
}

impl Room {
    pub fn new(id: &str, room_type: RoomType) -> Self {
        Self {
            id: id.to_string(),
            room_type,
            description: String::new(),
            created_at: 0,
            tile_count: 0,
            parent: None,
            children: Vec::new(),
            active: true,
            metadata: HashMap::new(),
        }
    }

    pub fn with_description(mut self, desc: &str) -> Self {
        self.description = desc.to_string();
        self
    }

    pub fn with_parent(mut self, parent: &str) -> Self {
        self.parent = Some(parent.to_string());
        self
    }

    pub fn path(&self) -> String {
        self.id.clone()
    }

    pub fn is_root(&self) -> bool { self.parent.is_none() }
}

/// Navigation history (breadcrumb trail).
#[derive(Debug, Clone, Default)]
pub struct NavHistory {
    entries: Vec<String>,
    current: usize,
}

impl NavHistory {
    pub fn new() -> Self { Self { entries: Vec::new(), current: 0 } }

    pub fn push(&mut self, room_id: &str) {
        if self.current < self.entries.len() {
            self.entries.truncate(self.current + 1);
        }
        self.entries.push(room_id.to_string());
        self.current = self.entries.len() - 1;
    }

    pub fn current(&self) -> Option<&str> {
        self.entries.get(self.current).map(|s| s.as_str())
    }

    pub fn back(&mut self) -> Option<&str> {
        if self.current > 0 {
            self.current -= 1;
            self.entries.get(self.current).map(|s| s.as_str())
        } else { None }
    }

    pub fn forward(&mut self) -> Option<&str> {
        if self.current + 1 < self.entries.len() {
            self.current += 1;
            self.entries.get(self.current).map(|s| s.as_str())
        } else { None }
    }

    pub fn len(&self) -> usize { self.entries.len() }
    pub fn is_empty(&self) -> bool { self.entries.is_empty() }
    pub fn can_go_back(&self) -> bool { self.current > 0 }
    pub fn can_go_forward(&self) -> bool { self.current + 1 < self.entries.len() }
}

/// Execution result from a room operation.
#[derive(Debug, Clone)]
pub struct ExecResult {
    pub success: bool,
    pub output: String,
    pub tiles_touched: usize,
    pub room_id: String,
}

/// The room engine — runtime for all rooms.
pub struct RoomEngine {
    rooms: HashMap<String, Room>,
    history: NavHistory,
    root_path: String,
}

impl RoomEngine {
    pub fn new(root_path: &str) -> Self {
        Self {
            rooms: HashMap::new(),
            history: NavHistory::new(),
            root_path: root_path.to_string(),
        }
    }

    /// Create a new room.
    pub fn create_room(&mut self, id: &str, room_type: RoomType) -> &Room {
        let room = Room::new(id, room_type);
        self.rooms.insert(id.to_string(), room);
        self.rooms.get(id).unwrap()
    }

    /// Create room with parent (nested rooms).
    pub fn create_child_room(&mut self, id: &str, parent_id: &str, room_type: RoomType) -> Result<&Room, String> {
        if !self.rooms.contains_key(parent_id) {
            return Err(format!("Parent room '{}' not found", parent_id));
        }
        let room = Room::new(id, room_type).with_parent(parent_id);
        self.rooms.get_mut(parent_id).unwrap().children.push(id.to_string());
        self.rooms.insert(id.to_string(), room);
        Ok(self.rooms.get(id).unwrap())
    }

    /// Navigate to a room.
    pub fn navigate(&mut self, id: &str) -> Result<&Room, String> {
        if !self.rooms.contains_key(id) {
            return Err(format!("Room '{}' not found", id));
        }
        self.history.push(id);
        Ok(self.rooms.get(id).unwrap())
    }

    /// Navigate back.
    pub fn navigate_back(&mut self) -> Result<Option<&Room>, String> {
        match self.history.back() {
            Some(id) => {
                let room_id = id.to_string();
                Ok(self.rooms.get(&room_id))
            }
            None => Ok(None),
        }
    }

    /// Get current room.
    pub fn current_room(&self) -> Option<&Room> {
        self.history.current().and_then(|id| self.rooms.get(id))
    }

    /// Execute a command in the current room.
    pub fn execute(&mut self, command: &str) -> ExecResult {
        let room_id = self.history.current().unwrap_or("none").to_string();
        // Simplified: parse command, return mock result
        let tiles_touched = if command.starts_with("query:") { 3 } else { 1 };
        let output = format!("Executed '{}' in room '{}'", command, room_id);
        ExecResult {
            success: true,
            output,
            tiles_touched,
            room_id,
        }
    }

    /// List all rooms.
    pub fn list_rooms(&self) -> Vec<&Room> {
        self.rooms.values().collect()
    }

    /// List rooms by type.
    pub fn list_by_type(&self, room_type: RoomType) -> Vec<&Room> {
        self.rooms.values().filter(|r| r.room_type == room_type).collect()
    }

    /// Get room by ID.
    pub fn get_room(&self, id: &str) -> Option<&Room> {
        self.rooms.get(id)
    }

    /// Get room count.
    pub fn room_count(&self) -> usize { self.rooms.len() }

    /// Deactivate a room.
    pub fn deactivate(&mut self, id: &str) -> bool {
        if let Some(room) = self.rooms.get_mut(id) {
            room.active = false;
            true
        } else { false }
    }

    /// Reactivate a room.
    pub fn activate(&mut self, id: &str) -> bool {
        if let Some(room) = self.rooms.get_mut(id) {
            room.active = true;
            true
        } else { false }
    }

    /// Get navigation history.
    pub fn history(&self) -> &NavHistory { &self.history }
}

impl Default for RoomEngine {
    fn default() -> Self { Self::new("/tmp/plato-data") }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_room() {
        let mut engine = RoomEngine::new("/test");
        let room = engine.create_room("math", RoomType::Learning);
        assert_eq!(room.id, "math");
        assert_eq!(room.room_type, RoomType::Learning);
        assert!(room.active);
    }

    #[test]
    fn test_create_child_room() {
        let mut engine = RoomEngine::new("/test");
        engine.create_room("root", RoomType::Application);
        let child = engine.create_child_room("sub", "root", RoomType::Learning).unwrap();
        assert_eq!(child.parent.as_deref(), Some("root"));
        let root = engine.get_room("root").unwrap();
        assert!(root.children.contains(&"sub".to_string()));
    }

    #[test]
    fn test_create_child_no_parent() {
        let mut engine = RoomEngine::new("/test");
        let result = engine.create_child_room("orphan", "nonexistent", RoomType::Learning);
        assert!(result.is_err());
    }

    #[test]
    fn test_navigate() {
        let mut engine = RoomEngine::new("/test");
        engine.create_room("a", RoomType::Learning);
        engine.create_room("b", RoomType::Application);
        engine.navigate("a").unwrap();
        assert_eq!(engine.current_room().unwrap().id, "a");
        engine.navigate("b").unwrap();
        assert_eq!(engine.current_room().unwrap().id, "b");
    }

    #[test]
    fn test_navigate_nonexistent() {
        let mut engine = RoomEngine::new("/test");
        let result = engine.navigate("ghost");
        assert!(result.is_err());
    }

    #[test]
    fn test_navigate_back() {
        let mut engine = RoomEngine::new("/test");
        engine.create_room("a", RoomType::Learning);
        engine.create_room("b", RoomType::Application);
        engine.navigate("a").unwrap();
        engine.navigate("b").unwrap();
        engine.navigate_back().unwrap();
        assert_eq!(engine.current_room().unwrap().id, "a");
    }

    #[test]
    fn test_navigate_forward() {
        let mut engine = RoomEngine::new("/test");
        engine.create_room("a", RoomType::Learning);
        engine.create_room("b", RoomType::Application);
        engine.navigate("a").unwrap();
        engine.navigate("b").unwrap();
        engine.navigate_back().unwrap();
        assert!(engine.history().can_go_forward());
    }

    #[test]
    fn test_execute() {
        let mut engine = RoomEngine::new("/test");
        engine.create_room("math", RoomType::Learning);
        engine.navigate("math").unwrap();
        let result = engine.execute("query: pythagorean theorem");
        assert!(result.success);
        assert_eq!(result.tiles_touched, 3);
        assert_eq!(result.room_id, "math");
    }

    #[test]
    fn test_list_rooms() {
        let mut engine = RoomEngine::new("/test");
        engine.create_room("a", RoomType::Learning);
        engine.create_room("b", RoomType::Application);
        engine.create_room("c", RoomType::System);
        assert_eq!(engine.list_rooms().len(), 3);
    }

    #[test]
    fn test_list_by_type() {
        let mut engine = RoomEngine::new("/test");
        engine.create_room("a", RoomType::Learning);
        engine.create_room("b", RoomType::Learning);
        engine.create_room("c", RoomType::System);
        assert_eq!(engine.list_by_type(RoomType::Learning).len(), 2);
        assert_eq!(engine.list_by_type(RoomType::System).len(), 1);
    }

    #[test]
    fn test_deactivate_activate() {
        let mut engine = RoomEngine::new("/test");
        engine.create_room("a", RoomType::Learning);
        assert!(engine.deactivate("a"));
        assert!(!engine.get_room("a").unwrap().active);
        assert!(engine.activate("a"));
        assert!(engine.get_room("a").unwrap().active);
    }

    #[test]
    fn test_deactivate_nonexistent() {
        let mut engine = RoomEngine::new("/test");
        assert!(!engine.deactivate("ghost"));
    }

    #[test]
    fn test_room_with_description() {
        let room = Room::new("math", RoomType::Learning).with_description("Math learning room");
        assert_eq!(room.description, "Math learning room");
    }

    #[test]
    fn test_room_with_parent() {
        let room = Room::new("sub", RoomType::Learning).with_parent("root");
        assert_eq!(room.parent.as_deref(), Some("root"));
        assert!(!room.is_root());
    }

    #[test]
    fn test_room_is_root() {
        let room = Room::new("root", RoomType::Application);
        assert!(room.is_root());
    }

    #[test]
    fn test_nav_history() {
        let mut nav = NavHistory::new();
        nav.push("a");
        nav.push("b");
        nav.push("c");
        assert_eq!(nav.current(), Some("c"));
        assert_eq!(nav.back(), Some("b"));
        assert!(nav.can_go_forward());
    }

    #[test]
    fn test_nav_history_empty() {
        let nav = NavHistory::new();
        assert!(nav.is_empty());
        assert!(nav.current().is_none());
    }

    #[test]
    fn test_nav_history_truncates_on_push() {
        let mut nav = NavHistory::new();
        nav.push("a");
        nav.push("b");
        nav.back(); // now at "a"
        nav.push("c"); // should truncate "b"
        assert!(!nav.can_go_forward());
        assert_eq!(nav.current(), Some("c"));
        assert_eq!(nav.len(), 2);
    }

    #[test]
    fn test_room_metadata() {
        let mut engine = RoomEngine::new("/test");
        engine.create_room("math", RoomType::Learning);
        engine.rooms.get_mut("math").unwrap().metadata.insert("level".to_string(), "advanced".to_string());
        assert_eq!(engine.get_room("math").unwrap().metadata.get("level").unwrap(), "advanced");
    }

    #[test]
    fn test_room_count() {
        let mut engine = RoomEngine::new("/test");
        assert_eq!(engine.room_count(), 0);
        engine.create_room("a", RoomType::Learning);
        assert_eq!(engine.room_count(), 1);
    }
}
