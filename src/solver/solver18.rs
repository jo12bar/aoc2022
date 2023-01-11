use std::{
    collections::{hash_map::Entry, HashMap},
    io::BufRead,
};

use color_eyre::eyre::{eyre, Context};
use itertools::Itertools;
use nalgebra_glm::IVec3;

#[derive(Debug, Default)]
pub struct Solver18;

impl super::ChallengeSolver for Solver18 {
    fn challenge_number(&self) -> crate::challenge::ChallengeNumber {
        18
    }

    fn solve_a(&mut self, input: &mut dyn BufRead) -> super::ChallengeSolverResult {
        let (world, world_bounds) =
            parse_input(input).wrap_err("Could not parse challenge input to a set of points")?;

        println!("world bounds: {world_bounds:#?}");

        let surface_area = calc_surface_area(&world);
        println!("surface area = {surface_area}");

        Ok(Box::new(()))
    }

    fn solve_b(&mut self, input: &mut dyn BufRead) -> super::ChallengeSolverResult {
        let (mut world, mut world_bounds) =
            parse_input(input).wrap_err("Could not parse challenge input to a set of points")?;

        // Fill in the world with:
        // - Voxel::Air, 1 cell outside of the world's current bounding box, increasing the world's
        //   bounding box by 1 cell in all directions
        // - Voxel::Vacuum in all positions not taken up my Voxel::Lava or Voxel::Air

        // First, preemptively grow the world_bounds by 1 in all directions
        world_bounds.x_max += 1;
        world_bounds.x_min -= 1;

        world_bounds.y_max += 1;
        world_bounds.y_min -= 1;

        world_bounds.z_max += 1;
        world_bounds.z_min -= 1;

        println!("world bounds: {world_bounds:#?}");

        // Reserve additional memory for the World HashMap to grow
        let voxel_count = (world_bounds.x_max - world_bounds.x_min + 1)
            * (world_bounds.y_max - world_bounds.y_min + 1)
            * (world_bounds.z_max - world_bounds.z_min + 1);
        let voxel_count: usize = voxel_count.try_into()?;
        world.reserve(voxel_count.saturating_sub(world.capacity()));

        // Iterate through all positions
        for ((x, y), z) in (world_bounds.x_min..=world_bounds.x_max)
            .cartesian_product(world_bounds.y_min..=world_bounds.y_max)
            .cartesian_product(world_bounds.z_min..=world_bounds.z_max)
        {
            let is_perimeter = (x == world_bounds.x_max || x == world_bounds.x_min)
                || (y == world_bounds.y_max || y == world_bounds.y_min)
                || (z == world_bounds.z_max || z == world_bounds.z_min);

            if is_perimeter {
                // If we're on the world's perimeter, insert Voxel::Air
                match world.entry([x, y, z].into()) {
                    Entry::Occupied(_) => unreachable!(
                        "A voxel already exists in perimeter position ({x}, {y}, {z}), \
                         which shouldn't be possible"
                    ),
                    Entry::Vacant(entry) => {
                        entry.insert(Voxel::Air);
                    }
                }
            } else {
                // Otherwise, insert Voxel::Vacuum if the entry is unoccupied
                world.entry([x, y, z].into()).or_insert(Voxel::Vacuum);
            }
        }

        // Begin simulating a cellular automaton.
        // Each loop, iterate through all Voxel::Vacuum's. If a Voxel::Vacuum is
        // adjacent to a Voxel::Air, turn it into a Voxel::Air.
        // Stop the loop when we detect that the no changes are made to the world
        // during a cycle.
        loop {
            let mut new_air_coords = Vec::new();

            #[rustfmt::skip]
            let neighbors: [IVec3; 6] = [
                [1, 0, 0].into(), [-1, 0, 0].into(),
                [0, 1, 0].into(), [0, -1, 0].into(),
                [0, 0, 1].into(), [0, 0, -1].into(),
            ];

            for (coord, _) in world.iter().filter(|(_, voxel)| **voxel == Voxel::Vacuum) {
                'inner: for neighbor in &neighbors {
                    let neighbor_coord = coord + neighbor;
                    if let Some(&Voxel::Air) = world.get(&neighbor_coord) {
                        new_air_coords.push(*coord);
                        break 'inner;
                    }
                }
            }

            for coord in &new_air_coords {
                world.insert(*coord, Voxel::Air);
            }

            if new_air_coords.is_empty() {
                break;
            }
        }

        // Finally, calculate the surface area of the droplet, excluding any droplet faces that
        // are adjacent to Voxel::Vacuum or Voxel::Lava.
        let surface_area = calc_surface_area(&world);
        println!("surface area = {surface_area}");

        Ok(Box::new(()))
    }
}

type World = HashMap<IVec3, Voxel>;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum Voxel {
    Lava,
    Air,
    Vacuum,
}

#[derive(Debug)]
struct WorldBounds {
    pub x_min: i32,
    pub x_max: i32,

    pub y_min: i32,
    pub y_max: i32,

    pub z_min: i32,
    pub z_max: i32,
}

fn parse_input(input: &mut dyn BufRead) -> color_eyre::Result<(World, WorldBounds)> {
    let mut points = World::new();

    let mut bounds = WorldBounds {
        x_min: i32::MAX,
        x_max: i32::MIN,

        y_min: i32::MAX,
        y_max: i32::MIN,

        z_min: i32::MAX,
        z_max: i32::MIN,
    };

    for line in input.lines() {
        let line = line.wrap_err("Could not read line from input file to string")?;

        let mut split = line.split(',');

        let (x_str, y_str, z_str) = (
            split
                .next()
                .ok_or_else(|| eyre!("Could not get x component from line {}", &line))?,
            split
                .next()
                .ok_or_else(|| eyre!("Could not get y component from line {}", &line))?,
            split
                .next()
                .ok_or_else(|| eyre!("Could not get z component from line {}", &line))?,
        );

        let (x, y, z): (i32, i32, i32) = (
            x_str
                .parse()
                .wrap_err_with(|| format!("Could not parse as x component: {x_str}"))?,
            y_str
                .parse()
                .wrap_err_with(|| format!("Could not parse as y component: {y_str}"))?,
            z_str
                .parse()
                .wrap_err_with(|| format!("Could not parse as z component: {z_str}"))?,
        );

        points.insert([x, y, z].into(), Voxel::Lava);

        bounds.x_min = bounds.x_min.min(x);
        bounds.x_max = bounds.x_max.max(x);

        bounds.y_min = bounds.y_min.min(y);
        bounds.y_max = bounds.y_max.max(y);

        bounds.z_min = bounds.z_min.min(z);
        bounds.z_max = bounds.z_max.max(z);
    }

    Ok((points, bounds))
}

fn calc_surface_area(world: &World) -> i32 {
    let mut area = 0;

    for (point, _) in world.iter().filter(|(_, voxel)| **voxel == Voxel::Lava) {
        #[rustfmt::skip]
        let neighbors: [IVec3; 6] = [
            [1, 0, 0].into(), [-1, 0, 0].into(),
            [0, 1, 0].into(), [0, -1, 0].into(),
            [0, 0, 1].into(), [0, 0, -1].into(),
        ];

        for neighbor in &neighbors {
            let coord = point + neighbor;
            let neighbor_voxel = world.get(&coord);

            // Only include empty adjacent integer cells in the surface area calculation
            // (OR cells that contain only Voxel::Air, and never Voxel::Lava)
            if neighbor_voxel.is_none() || matches!(neighbor_voxel, Some(Voxel::Air)) {
                area += 1;
            }
        }
    }

    area
}
