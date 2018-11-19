//
// Created by John Nguyen on 04.11.18.
//

#ifndef NEWTON_NEWTON_H
#define NEWTON_NEWTON_H

#include <stdint.h>

/**
 *  A simple wrapper struct to encapsulate
 *  point data.
 */
struct NewtonPoint {
    int32_t x;
    int32_t y;
};

/**
 *  A opaque struct representing the
 *  gravitational field.
 */
struct field;

/**
 *  Allocates a new field instance with the given gravitational
 *  constant, solar mass, min and max effective distance.
 */
struct field *newton_new_field(double g, double solar_mass, double min_dist, double max_dist);

/**
 *  Destroys the field instance referred
 *  to by the given pointer.
 */
void newton_destroy_field(struct field *field);

/**
 *  Creates a new body instance with the
 *  given properties (id, mass, position, velocity)
 *  and adds it to the field.
 */
void newton_add_body(struct field *field, uint32_t id, double mass, int32_t x, int32_t y, double dx, double dy);

/**
 *  Generates a radial distribution of num_bodies between
 *  min_dist and max_dist from a central point. These are
 *  assigned to the field.
 */
void newton_distribute_bodies(struct field *field, uint32_t num_bodies, uint32_t min_dist, uint32_t max_dist, double dy);

 /**
  *  Advances the field state by a single step.
  */
 void newton_step(struct field *field);

 /**
  *  Returns the coordinate of the body with the
  *  given id, if it exists, else the origin.
  */
struct NewtonPoint newton_body_pos(const struct field *field, uint32_t id);

#endif //NEWTON_NEWTON_H
