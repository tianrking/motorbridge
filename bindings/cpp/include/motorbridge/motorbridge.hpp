#pragma once

#include <array>
#include <cstdint>
#include <memory>
#include <optional>
#include <stdexcept>
#include <string>
#include <utility>

extern "C" {
#include "motor_abi.h"
}

namespace motorbridge {

class Error : public std::runtime_error {
 public:
  explicit Error(const std::string& message) : std::runtime_error(message) {}
};

inline std::string last_error_message() {
  const char* msg = motor_last_error_message();
  if (!msg) return "unknown error";
  return std::string(msg);
}

inline void check_rc(int32_t rc, const char* what) {
  if (rc == 0) return;
  throw Error(std::string(what) + " failed: " + last_error_message());
}

enum class Mode : uint32_t {
  MIT = 1,
  POS_VEL = 2,
  VEL = 3,
  FORCE_POS = 4,
};

struct State {
  uint8_t can_id;
  uint16_t arbitration_id;
  uint8_t status_code;
  float pos;
  float vel;
  float torq;
  float t_mos;
  float t_rotor;
};

struct RegisterSpec {
  uint8_t rid;
  const char* variable;
  const char* description;
  const char* data_type;
  const char* access;
  const char* range;
};

inline constexpr uint8_t RID_UV_VALUE = 0;
inline constexpr uint8_t RID_KT_VALUE = 1;
inline constexpr uint8_t RID_OT_VALUE = 2;
inline constexpr uint8_t RID_OC_VALUE = 3;
inline constexpr uint8_t RID_ACC = 4;
inline constexpr uint8_t RID_DEC = 5;
inline constexpr uint8_t RID_MAX_SPD = 6;
inline constexpr uint8_t RID_MST_ID = 7;
inline constexpr uint8_t RID_ESC_ID = 8;
inline constexpr uint8_t RID_TIMEOUT = 9;
inline constexpr uint8_t RID_CTRL_MODE = 10;
inline constexpr uint8_t RID_PMAX = 21;
inline constexpr uint8_t RID_VMAX = 22;
inline constexpr uint8_t RID_TMAX = 23;

inline constexpr std::array<RegisterSpec, 26> DAMIAO_RW_REGISTERS = {{
    {0, "UV_Value", "Under-voltage protection value", "f32", "RW", "(10.0, 3.4E38]"},
    {1, "KT_Value", "Torque coefficient", "f32", "RW", "[0.0, 3.4E38]"},
    {2, "OT_Value", "Over-temperature protection value", "f32", "RW", "[80.0, 200)"},
    {3, "OC_Value", "Over-current protection value", "f32", "RW", "(0.0, 1.0)"},
    {4, "ACC", "Acceleration", "f32", "RW", "(0.0, 3.4E38)"},
    {5, "DEC", "Deceleration", "f32", "RW", "[-3.4E38, 0.0)"},
    {6, "MAX_SPD", "Maximum speed", "f32", "RW", "(0.0, 3.4E38]"},
    {7, "MST_ID", "Feedback ID", "u32", "RW", "[0, 0x7FF]"},
    {8, "ESC_ID", "Receive ID", "u32", "RW", "[0, 0x7FF]"},
    {9, "TIMEOUT", "Timeout alarm time", "u32", "RW", "[0, 2^32-1]"},
    {10, "CTRL_MODE", "Control mode", "u32", "RW", "[1, 4]"},
    {21, "PMAX", "Position mapping range", "f32", "RW", "(0.0, 3.4E38]"},
    {22, "VMAX", "Speed mapping range", "f32", "RW", "(0.0, 3.4E38]"},
    {23, "TMAX", "Torque mapping range", "f32", "RW", "(0.0, 3.4E38]"},
    {24, "I_BW", "Current loop control bandwidth", "f32", "RW", "[100.0, 10000.0]"},
    {25, "KP_ASR", "Speed loop Kp", "f32", "RW", "[0.0, 3.4E38]"},
    {26, "KI_ASR", "Speed loop Ki", "f32", "RW", "[0.0, 3.4E38]"},
    {27, "KP_APR", "Position loop Kp", "f32", "RW", "[0.0, 3.4E38]"},
    {28, "KI_APR", "Position loop Ki", "f32", "RW", "[0.0, 3.4E38]"},
    {29, "OV_Value", "Over-voltage protection value", "f32", "RW", "TBD"},
    {30, "GREF", "Gear torque efficiency", "f32", "RW", "(0.0, 1.0]"},
    {31, "Deta", "Speed loop damping coefficient", "f32", "RW", "[1.0, 30.0]"},
    {32, "V_BW", "Speed loop filter bandwidth", "f32", "RW", "(0.0, 500.0)"},
    {33, "IQ_c1", "Current loop enhancement coefficient", "f32", "RW", "[100.0, 10000.0]"},
    {34, "VL_c1", "Speed loop enhancement coefficient", "f32", "RW", "(0.0, 10000.0]"},
    {35, "can_br", "CAN baud rate code", "u32", "RW", "[0, 4]"},
}};

inline constexpr std::array<uint8_t, 11> DAMIAO_HIGH_IMPACT_RIDS = {
    21, 22, 23, 25, 26, 27, 28, 4, 5, 6, 9};
inline constexpr std::array<uint8_t, 4> DAMIAO_PROTECTION_RIDS = {0, 2, 3, 29};

inline const RegisterSpec* get_damiao_register_spec(uint8_t rid) {
  for (const auto& spec : DAMIAO_RW_REGISTERS) {
    if (spec.rid == rid) return &spec;
  }
  return nullptr;
}

class Controller;

namespace detail {
struct ControllerHandle {
  MotorController* ptr;

  explicit ControllerHandle(MotorController* p) : ptr(p) {}

  ~ControllerHandle() {
    if (ptr) motor_controller_free(ptr);
  }
};
}  // namespace detail

class Motor {
 public:
  Motor() = delete;
  ~Motor() {
    if (ptr_) motor_handle_free(ptr_);
  }

  Motor(Motor&& other) noexcept : controller_(std::move(other.controller_)), ptr_(other.ptr_) {
    other.ptr_ = nullptr;
  }

  Motor& operator=(Motor&& other) noexcept {
    if (this == &other) return *this;
    if (ptr_) motor_handle_free(ptr_);
    controller_ = std::move(other.controller_);
    ptr_ = other.ptr_;
    other.ptr_ = nullptr;
    return *this;
  }

  Motor(const Motor&) = delete;
  Motor& operator=(const Motor&) = delete;

  void enable() { check_rc(motor_handle_enable(ptr_), "enable"); }
  void disable() { check_rc(motor_handle_disable(ptr_), "disable"); }
  void clear_error() { check_rc(motor_handle_clear_error(ptr_), "clear_error"); }
  void set_zero_position() {
    check_rc(motor_handle_set_zero_position(ptr_), "set_zero_position");
  }
  void ensure_mode(Mode mode, uint32_t timeout_ms = 1000) {
    check_rc(motor_handle_ensure_mode(ptr_, static_cast<uint32_t>(mode), timeout_ms), "ensure_mode");
  }
  void send_mit(float pos, float vel, float kp, float kd, float tau) {
    check_rc(motor_handle_send_mit(ptr_, pos, vel, kp, kd, tau), "send_mit");
  }
  void send_pos_vel(float pos, float vlim) {
    check_rc(motor_handle_send_pos_vel(ptr_, pos, vlim), "send_pos_vel");
  }
  void send_vel(float vel) { check_rc(motor_handle_send_vel(ptr_, vel), "send_vel"); }
  void send_force_pos(float pos, float vlim, float ratio) {
    check_rc(motor_handle_send_force_pos(ptr_, pos, vlim, ratio), "send_force_pos");
  }
  void request_feedback() { check_rc(motor_handle_request_feedback(ptr_), "request_feedback"); }
  void set_can_timeout_ms(uint32_t timeout_ms) {
    check_rc(motor_handle_set_can_timeout_ms(ptr_, timeout_ms), "set_can_timeout_ms");
  }
  void store_parameters() {
    check_rc(motor_handle_store_parameters(ptr_), "store_parameters");
  }

  void write_register_f32(uint8_t rid, float value) {
    check_rc(motor_handle_write_register_f32(ptr_, rid, value), "write_register_f32");
  }
  void write_register_u32(uint8_t rid, uint32_t value) {
    check_rc(motor_handle_write_register_u32(ptr_, rid, value), "write_register_u32");
  }
  float get_register_f32(uint8_t rid, uint32_t timeout_ms = 1000) {
    float out = 0.0f;
    check_rc(motor_handle_get_register_f32(ptr_, rid, timeout_ms, &out), "get_register_f32");
    return out;
  }
  uint32_t get_register_u32(uint8_t rid, uint32_t timeout_ms = 1000) {
    uint32_t out = 0;
    check_rc(motor_handle_get_register_u32(ptr_, rid, timeout_ms, &out), "get_register_u32");
    return out;
  }

  std::optional<State> get_state() const {
    MotorState st{};
    check_rc(motor_handle_get_state(ptr_, &st), "get_state");
    if (!st.has_value) return std::nullopt;
    return State{
        st.can_id, st.arbitration_id, st.status_code, st.pos, st.vel, st.torq, st.t_mos, st.t_rotor};
  }

 private:
  friend class Controller;

  Motor(std::shared_ptr<detail::ControllerHandle> controller, MotorHandle* ptr)
      : controller_(std::move(controller)), ptr_(ptr) {}

  std::shared_ptr<detail::ControllerHandle> controller_;
  MotorHandle* ptr_;
};

class Controller {
 public:
  explicit Controller(const std::string& channel = "can0") {
    MotorController* raw = motor_controller_new_socketcan(channel.c_str());
    if (!raw) {
      throw Error("new_socketcan failed: " + last_error_message());
    }
    handle_ = std::make_shared<detail::ControllerHandle>(raw);
  }

  Controller(Controller&&) noexcept = default;
  Controller& operator=(Controller&&) noexcept = default;
  Controller(const Controller&) = delete;
  Controller& operator=(const Controller&) = delete;
  ~Controller() = default;

  void enable_all() { check_rc(motor_controller_enable_all(handle_->ptr), "enable_all"); }
  void disable_all() { check_rc(motor_controller_disable_all(handle_->ptr), "disable_all"); }
  void poll_feedback_once() {
    check_rc(motor_controller_poll_feedback_once(handle_->ptr), "poll_feedback_once");
  }
  void shutdown() { check_rc(motor_controller_shutdown(handle_->ptr), "shutdown"); }
  void close_bus() { check_rc(motor_controller_close_bus(handle_->ptr), "close_bus"); }

  Motor add_damiao_motor(uint16_t motor_id, uint16_t feedback_id, const std::string& model) {
    MotorHandle* m = motor_controller_add_damiao_motor(handle_->ptr, motor_id, feedback_id, model.c_str());
    if (!m) {
      throw Error("add_damiao_motor failed: " + last_error_message());
    }
    return Motor(handle_, m);
  }

 private:
  std::shared_ptr<detail::ControllerHandle> handle_;
};

}  // namespace motorbridge

