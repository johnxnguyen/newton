//
// Created by John Nguyen on 04.11.18.
//

#ifndef NEWTON_NEWTON_H
#define NEWTON_NEWTON_H

#include <stdint.h>

/**
 *  A opaque struct representing the
 *  gravitational field.
 */
struct field;

/**
 *  Allcoates a new field instance with
 *  the given gravitational constant and solar mass.
 */
struct field *newton_new_field(double g, double solar_mass);

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
  *  Advances the field state by a single step.
  */
 void newton_step(struct field *field);

 /**
  *  Returns the x coordinate of the body with the
  *  given id, if it exists, else the max value of
  *  int32_t.
  */
int32_t newton_body_x_pos(const struct field *field, uint32_t id);

/**
*  Returns the y coordinate of the body with the
*  given id, if it exists, else the max value of
*  int32_t.
*/
int32_t newton_body_y_pos(const struct field *field, uint32_t id);

#endif //NEWTON_NEWTON_H
