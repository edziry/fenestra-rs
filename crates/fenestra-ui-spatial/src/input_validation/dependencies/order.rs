//! Stable Kahn ordering and iterative cyclic-component attribution.

use std::cmp::Reverse;
use std::collections::BinaryHeap;

use super::DependencyUnitPlan;

pub(super) fn stable_order(units: &[DependencyUnitPlan]) -> Result<Vec<usize>, u32> {
    let mut outgoing = vec![Vec::new(); units.len()];
    let mut indegree = units
        .iter()
        .map(|unit| unit.incoming.len())
        .collect::<Vec<_>>();
    for (consumer, unit) in units.iter().enumerate() {
        for &producer in &unit.incoming {
            outgoing[producer].push(consumer);
        }
    }

    let mut ready = BinaryHeap::new();
    for (index, &count) in indegree.iter().enumerate() {
        if count == 0 {
            ready.push(Reverse(index));
        }
    }

    let mut order = Vec::with_capacity(units.len());
    while let Some(Reverse(producer)) = ready.pop() {
        order.push(producer);
        for &consumer in &outgoing[producer] {
            indegree[consumer] = indegree[consumer]
                .checked_sub(1)
                .expect("each unique edge decrements indegree once");
            if indegree[consumer] == 0 {
                ready.push(Reverse(consumer));
            }
        }
    }

    if order.len() == units.len() {
        return Ok(order);
    }

    let unresolved = indegree.iter().map(|&count| count != 0).collect::<Vec<_>>();
    let cycle = smallest_cyclic_component(units, &outgoing, &unresolved);
    Err(units[cycle].ordinal)
}

fn smallest_cyclic_component(
    units: &[DependencyUnitPlan],
    outgoing: &[Vec<usize>],
    unresolved: &[bool],
) -> usize {
    let finish = finish_order(outgoing, unresolved);
    let mut assigned = vec![false; units.len()];
    let mut best = None;

    for start in finish.into_iter().rev() {
        if !unresolved[start] || assigned[start] {
            continue;
        }

        let mut component = Vec::new();
        let mut stack = vec![start];
        assigned[start] = true;
        while let Some(node) = stack.pop() {
            component.push(node);
            for &producer in units[node].incoming.iter().rev() {
                if unresolved[producer] && !assigned[producer] {
                    assigned[producer] = true;
                    stack.push(producer);
                }
            }
        }

        let minimum = *component
            .iter()
            .min()
            .expect("every component contains its start");
        let cyclic = component.len() > 1 || units[minimum].incoming.contains(&minimum);
        if cyclic && best.is_none_or(|current| minimum < current) {
            best = Some(minimum);
        }
    }

    best.expect("an unresolved finite directed graph contains a cyclic component")
}

fn finish_order(outgoing: &[Vec<usize>], unresolved: &[bool]) -> Vec<usize> {
    let mut visited = vec![false; outgoing.len()];
    let mut finish = Vec::with_capacity(outgoing.len());

    for start in 0..outgoing.len() {
        if !unresolved[start] || visited[start] {
            continue;
        }
        visited[start] = true;
        let mut stack = vec![(start, 0usize)];

        while let Some(&(node, next_index)) = stack.last() {
            if next_index == outgoing[node].len() {
                stack.pop();
                finish.push(node);
                continue;
            }

            let next = outgoing[node][next_index];
            stack.last_mut().expect("the DFS frame remains present").1 += 1;
            if unresolved[next] && !visited[next] {
                visited[next] = true;
                stack.push((next, 0));
            }
        }
    }

    finish
}
