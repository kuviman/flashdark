use super::*;

pub struct SpatialMap {
    max_radius: R32,
    cell_size: R32,
    cells: HashMap<Vec2<i32>, HashSet<Id>>,
    location: HashMap<Id, Vec2<i32>>,
}

impl SpatialMap {
    pub fn new(cell_size: R32) -> Self {
        Self {
            max_radius: R32::ZERO,
            cell_size,
            cells: default(),
            location: default(),
        }
    }
    pub fn insert(&mut self, id: Id, position: Vec2<R32>, radius: R32) {
        let position = position / self.cell_size;
        let radius = radius / self.cell_size;
        self.remove(id);
        let cell = position.map(|x| x.floor().raw() as i32);
        self.cells.entry(cell).or_default().insert(id);
        self.location.insert(id, cell);
        self.max_radius = max(self.max_radius, radius);
    }
    pub fn remove(&mut self, id: Id) -> bool {
        match self.location.remove(&id) {
            Some(cell) => {
                assert!(self.cells.get_mut(&cell).unwrap().remove(&id));
                true
            }
            None => false,
        }
    }
    pub fn lookup(&self, aabb: AABB<R32>) -> impl Iterator<Item = Id> + '_ {
        let aabb = aabb.map(|x| x / self.cell_size);
        let aabb = aabb.extend_uniform(self.max_radius);
        let aabb = AABB {
            x_min: aabb.x_min.floor().raw() as i32,
            y_min: aabb.y_min.floor().raw() as i32,
            x_max: aabb.x_max.ceil().raw() as i32,
            y_max: aabb.y_max.ceil().raw() as i32,
        };
        aabb.points().flat_map(move |cell| {
            self.cells
                .get(&cell)
                .into_iter()
                .flat_map(move |ids| ids.iter().copied())
        })
    }
}
