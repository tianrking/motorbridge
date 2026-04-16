#ifndef MOTOR_ABI_H
#define MOTOR_ABI_H

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct MotorController MotorController;
typedef struct MotorHandle MotorHandle;

typedef struct MotorState {
  int32_t has_value;
  uint8_t can_id;
  uint32_t arbitration_id;
  uint8_t status_code;
  float pos;
  float vel;
  float torq;
  float t_mos;
  float t_rotor;
} MotorState;

// Unified units across all vendors:
// - position: rad
// - velocity: rad/s
// - torque: Nm
//
// Unified mode IDs for motor_handle_ensure_mode:
// - 1: MIT
// - 2: POS_VEL
// - 3: VEL
// - 4: FORCE_POS
//
// Vendor support in current ABI:
// - Damiao
// - Hexfellow (CAN-FD transport via motor_controller_new_socketcanfd)
// - RobStride
// - MyActuator
// - HighTorque (native ht_can v1.5.5)

const char* motor_last_error_message(void);

MotorController* motor_controller_new_socketcan(const char* channel);
MotorController* motor_controller_new_socketcanfd(const char* channel);
MotorController* motor_controller_new_dm_serial(const char* serial_port, uint32_t baud);
void motor_controller_free(MotorController* controller);
int32_t motor_controller_poll_feedback_once(MotorController* controller);
int32_t motor_controller_enable_all(MotorController* controller);
int32_t motor_controller_disable_all(MotorController* controller);
int32_t motor_controller_shutdown(MotorController* controller);
int32_t motor_controller_close_bus(MotorController* controller);

MotorHandle* motor_controller_add_damiao_motor(MotorController* controller, uint16_t motor_id, uint16_t feedback_id, const char* model);
MotorHandle* motor_controller_add_hexfellow_motor(MotorController* controller, uint16_t motor_id, uint16_t feedback_id, const char* model);
MotorHandle* motor_controller_add_myactuator_motor(MotorController* controller, uint16_t motor_id, uint16_t feedback_id, const char* model);
MotorHandle* motor_controller_add_robstride_motor(MotorController* controller, uint16_t motor_id, uint16_t feedback_id, const char* model);
MotorHandle* motor_controller_add_hightorque_motor(MotorController* controller, uint16_t motor_id, uint16_t feedback_id, const char* model);
void motor_handle_free(MotorHandle* motor);

int32_t motor_handle_enable(MotorHandle* motor);
int32_t motor_handle_disable(MotorHandle* motor);
int32_t motor_handle_clear_error(MotorHandle* motor);
int32_t motor_handle_set_zero_position(MotorHandle* motor);
int32_t motor_handle_ensure_mode(MotorHandle* motor, uint32_t mode, uint32_t timeout_ms);

int32_t motor_handle_send_mit(MotorHandle* motor, float target_position, float target_velocity, float stiffness, float damping, float feedforward_torque);
int32_t motor_handle_send_pos_vel(MotorHandle* motor, float target_position, float velocity_limit);
int32_t motor_handle_send_vel(MotorHandle* motor, float target_velocity);
int32_t motor_handle_send_force_pos(MotorHandle* motor, float target_position, float velocity_limit, float torque_limit_ratio);

int32_t motor_handle_store_parameters(MotorHandle* motor);
int32_t motor_handle_request_feedback(MotorHandle* motor);
int32_t motor_handle_set_can_timeout_ms(MotorHandle* motor, uint32_t timeout_ms);

int32_t motor_handle_write_register_f32(MotorHandle* motor, uint8_t rid, float value);
int32_t motor_handle_write_register_u32(MotorHandle* motor, uint8_t rid, uint32_t value);
int32_t motor_handle_get_register_f32(MotorHandle* motor, uint8_t rid, uint32_t timeout_ms, float* out_value);
int32_t motor_handle_get_register_u32(MotorHandle* motor, uint8_t rid, uint32_t timeout_ms, uint32_t* out_value);

int32_t motor_handle_robstride_ping(MotorHandle* motor, uint8_t* out_device_id, uint8_t* out_responder_id);
int32_t motor_handle_robstride_set_device_id(MotorHandle* motor, uint8_t new_device_id);
int32_t motor_handle_robstride_write_param_i8(MotorHandle* motor, uint16_t param_id, int8_t value);
int32_t motor_handle_robstride_write_param_u8(MotorHandle* motor, uint16_t param_id, uint8_t value);
int32_t motor_handle_robstride_write_param_u16(MotorHandle* motor, uint16_t param_id, uint16_t value);
int32_t motor_handle_robstride_write_param_u32(MotorHandle* motor, uint16_t param_id, uint32_t value);
int32_t motor_handle_robstride_write_param_f32(MotorHandle* motor, uint16_t param_id, float value);
int32_t motor_handle_robstride_get_param_i8(MotorHandle* motor, uint16_t param_id, uint32_t timeout_ms, int8_t* out_value);
int32_t motor_handle_robstride_get_param_u8(MotorHandle* motor, uint16_t param_id, uint32_t timeout_ms, uint8_t* out_value);
int32_t motor_handle_robstride_get_param_u16(MotorHandle* motor, uint16_t param_id, uint32_t timeout_ms, uint16_t* out_value);
int32_t motor_handle_robstride_get_param_u32(MotorHandle* motor, uint16_t param_id, uint32_t timeout_ms, uint32_t* out_value);
int32_t motor_handle_robstride_get_param_f32(MotorHandle* motor, uint16_t param_id, uint32_t timeout_ms, float* out_value);

int32_t motor_handle_get_state(MotorHandle* motor, MotorState* out_state);

int32_t motor_handle_damiao_get_param_f32(MotorHandle* motor, uint16_t param_id, uint32_t timeout_ms, float* out_value);
int32_t motor_handle_damiao_get_param_u32(MotorHandle* motor, uint16_t param_id, uint32_t timeout_ms, uint32_t* out_value);
int32_t motor_handle_damiao_write_param_f32(MotorHandle* motor, uint16_t param_id, float value);
int32_t motor_handle_damiao_write_param_u32(MotorHandle* motor, uint16_t param_id, uint32_t value);

#ifdef __cplusplus
}
#endif

#endif
