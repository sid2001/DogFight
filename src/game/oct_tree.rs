use bevy::prelude::*;
use std::sync::{Arc, RwLock};

const ROOT_HALF_LENGTH: f32 = 128.;
const ROOT_CENTER: Vec3 = Vec3::splat(0.);
const SPLIT_POINT: u32 = 5;
const MINIMUM_HL: f32 = 0.0625;

pub struct OctTreePlugin;
impl Plugin for OctTreePlugin {
    fn build(&self, app: &mut App) {
        // app.add_systems(PostStartup, print_landscape);
    }
}

pub struct ChildrenMask(Vec<Vec3>);

impl Default for ChildrenMask {
    fn default() -> Self {
        // let mut vc = Vec::<Vec3>::new();
        let vc = Vec::<Vec3>::from([
            Vec3::new(1., 1., 1.),
            Vec3::new(-1., 1., 1.),
            Vec3::new(-1., 1., -1.),
            Vec3::new(1., 1., -1.),
            Vec3::new(1., -1., 1.),
            Vec3::new(-1., -1., 1.),
            Vec3::new(-1., -1., -1.),
            Vec3::new(1., -1., -1.),
        ]);
        ChildrenMask(vc)
    }
}

#[derive(Clone, Copy)]
pub struct NodeEntities {
    pub entity: Entity,
    pub radius: f32,
    pub center: Vec3,
}

#[derive(Clone)]
pub struct OctNode {
    pub children_mask: Arc<ChildrenMask>,
    pub children: Option<Vec<Arc<RwLock<OctNode>>>>,
    pub is_leaf_node: bool,
    pub half_length: f32,
    pub center: Vec3,
    // pub parent: Option<Arc<OctNode>>,
    pub split_point: u32,
    pub ticks: u32,
    pub min_hl: f32,
    pub objects: Vec<NodeEntities>,
}

type Root = OctNode;

pub struct OctTree {
    pub root: Box<Root>,
    children_mask: Arc<ChildrenMask>,
    pub pending_insertions: RwLock<Vec<NodeEntities>>,
}

impl OctNode {
    fn new_center(ref_point: &Vec3, mask: Vec3, hl: f32) -> Vec3 {
        Vec3::new(
            ref_point.x + mask.x * hl,
            ref_point.y + mask.y * hl,
            ref_point.z + mask.z * hl,
        )
    }

    fn on_parent(&self, obj: &NodeEntities) -> bool {
        let r = obj.radius;
        let Vec3 { x, y, z } = self.center - obj.center;
        if x.abs() <= r || y.abs() <= r || z.abs() <= r {
            true
        } else {
            false
        }
    }

    fn is_bounding(&self, obj: &NodeEntities) -> bool {
        let hl = self.half_length;
        let r = obj.radius;
        let Vec3 { x, y, z } = self.center - obj.center;
        if x.abs() + r < hl && y.abs() + r < hl && z.abs() + r < hl {
            true
        } else {
            false
        }
    }

    fn rearrange_objects_to_children(&mut self) {
        let mut parent_object = Vec::<NodeEntities>::new();
        for obj in &self.objects {
            if self.on_parent(obj) {
                parent_object.push(obj.clone());
            } else {
                let children = self.children.as_mut().unwrap();
                for i in 0..8 {
                    let mut child = children[i].as_ref().write().unwrap();
                    if child.is_bounding(obj) {
                        child.objects.push(obj.clone());
                    }
                }
            }
        }
        self.objects = parent_object;
    }

    fn insert(&mut self, obj: &NodeEntities) {
        if self.is_leaf_node {
            if self.objects.len() < SPLIT_POINT as usize || self.half_length <= MINIMUM_HL {
                self.objects.push(obj.clone());
            } else {
                self.is_leaf_node = false;
                let _ = self.build_children();
                self.rearrange_objects_to_children();
                self.insert(obj);
            }
        } else {
            if self.on_parent(obj) {
                self.objects.push(obj.clone());
            } else {
                for i in 0..8 {
                    let children = self.children.as_mut().unwrap();
                    let mut child = children[i].as_ref().write().unwrap();
                    if child.is_bounding(obj) {
                        child.insert(obj);
                    }
                }
            }
        }
    }

    fn build_children(&mut self) -> Result<(), String> {
        if self.half_length == self.min_hl {
            Err(String::from("minimum division limit reached!!"))
        } else {
            self.is_leaf_node = false;
            let mut children: Vec<Arc<RwLock<OctNode>>> = Vec::new();
            for i in 0..8 as usize {
                let mask = self.children_mask.0[i];
                let half_length = self.half_length / 2.;
                // let parent = Some(Arc::new(self));
                let center = OctNode::new_center(&self.center, mask, half_length);
                let node = OctNode::new(
                    half_length,
                    center,
                    // parent,
                    SPLIT_POINT,
                    Arc::clone(&self.children_mask),
                    MINIMUM_HL,
                );
                children.push(Arc::new(RwLock::new(node)));
            }
            self.children = Some(children);
            Ok(())
        }
    }
}

impl OctTree {
    pub fn update_tree(&self) {
        // if self.pending_insertions
        todo!();
    }

    pub fn build_tree(&mut self) {
        for obj in self.pending_insertions.read().unwrap().iter() {
            self.root.insert(obj);
        }
    }
    pub fn get_local_objects(&self, center: Vec3, radius: f32) -> Vec<Entity> {
        let result: Vec<Entity> = Vec::new();
        todo!();
        result
    }
}

impl OctNode {
    fn new(
        half_length: f32,
        center: Vec3,
        // parent: Option<Arc<OctNode>>,
        split_point: u32,
        children_mask: Arc<ChildrenMask>,
        min_hl: f32,
    ) -> Self {
        Self {
            half_length,
            center,
            // parent,
            children_mask,
            children: None,
            is_leaf_node: true,
            split_point,
            ticks: 10,
            min_hl,
            objects: Vec::new(),
        }
    }
}

impl Default for OctTree {
    fn default() -> Self {
        let children_mask = Arc::new(ChildrenMask::default());
        let node = OctNode::new(
            ROOT_HALF_LENGTH,
            ROOT_CENTER,
            // None,
            SPLIT_POINT,
            children_mask.clone(),
            MINIMUM_HL,
        );
        let root = Box::new(node);

        Self {
            root,
            children_mask,
            pending_insertions: RwLock::new(Vec::<NodeEntities>::with_capacity(
                SPLIT_POINT as usize,
            )),
        }
    }
}
