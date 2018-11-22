//
// Created by John Nguyen on 04.11.18.
//

#ifndef NEWTON_NEWTON_H
#define NEWTON_NEWTON_H

#include <stdint.h>

// A simple wrapper struct to encapsulate point data.
struct NewtonPoint {
    float x;
    float y;
};

// An opaque struct representing the environment.
//
struct environment;

// Allocates a new Environment instance.
//
struct environment *newton_new_environment();

// Destroys the Environment instance referred to by the given pointer.
//
void newton_destroy_environment(struct environment *environment);

// Generates a radial distribution of bodies around a central point.
//
void newton_distribute_bodies(struct environment *environment, uint32_t num_bodies, float min_dist, float max_dist, float dy);

// Advances teh field state by a single step.
//
void newton_step(struct environment *environment);

// Returns the coordinates of the body with the given ID, if it exists,
// else the origin is returned.
struct NewtonPoint newton_body_pos(const struct environment *environment, uint32_t id);

#endif //NEWTON_NEWTON_H
