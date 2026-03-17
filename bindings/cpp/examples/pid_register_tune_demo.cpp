#include <cstdint>
#include <exception>
#include <iostream>
#include <optional>
#include <string>

#include "motorbridge/motorbridge.hpp"

struct Options {
  std::string channel = "can0";
  std::string model = "4340P";
  uint16_t motor_id = 0x01;
  uint16_t feedback_id = 0x11;
  uint32_t timeout_ms = 1000;
  int store = 0;

  std::optional<float> kp_asr;
  std::optional<float> ki_asr;
  std::optional<float> kp_apr;
  std::optional<float> ki_apr;
  std::optional<float> pmax;
  std::optional<float> vmax;
  std::optional<float> tmax;
  std::optional<float> acc;
  std::optional<float> dec;
  std::optional<float> max_spd;
};

static void print_help() {
  std::cout
      << "pid_register_tune_demo: tune Damiao PID/high-impact registers via C++ wrapper\\n"
      << "Usage:\\n"
      << "  ./pid_register_tune_demo --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \\\\n"
      << "    [--kp-asr 20] [--ki-asr 0.2] [--kp-apr 30] [--ki-apr 0.1] \\\\n"
      << "    [--pmax 12.5] [--vmax 45] [--tmax 18] [--acc 30] [--dec -30] [--max-spd 30] [--store 1]\\n\\n"
      << "Reads current values first, writes provided fields only, then reads back.\\n";
}

static bool parse_args(int argc, char** argv, Options& o) {
  for (int i = 1; i < argc; ++i) {
    std::string k = argv[i];
    if (k == "--help") {
      print_help();
      return false;
    }
    if (i + 1 >= argc) {
      std::cerr << "missing value for " << k << "\\n";
      return false;
    }
    std::string v = argv[++i];
    try {
      if (k == "--channel") o.channel = v;
      else if (k == "--model") o.model = v;
      else if (k == "--motor-id") o.motor_id = static_cast<uint16_t>(std::stoul(v, nullptr, 0));
      else if (k == "--feedback-id") o.feedback_id = static_cast<uint16_t>(std::stoul(v, nullptr, 0));
      else if (k == "--timeout-ms") o.timeout_ms = static_cast<uint32_t>(std::stoul(v, nullptr, 0));
      else if (k == "--store") o.store = std::stoi(v);
      else if (k == "--kp-asr") o.kp_asr = std::stof(v);
      else if (k == "--ki-asr") o.ki_asr = std::stof(v);
      else if (k == "--kp-apr") o.kp_apr = std::stof(v);
      else if (k == "--ki-apr") o.ki_apr = std::stof(v);
      else if (k == "--pmax") o.pmax = std::stof(v);
      else if (k == "--vmax") o.vmax = std::stof(v);
      else if (k == "--tmax") o.tmax = std::stof(v);
      else if (k == "--acc") o.acc = std::stof(v);
      else if (k == "--dec") o.dec = std::stof(v);
      else if (k == "--max-spd") o.max_spd = std::stof(v);
      else {
        std::cerr << "unknown arg: " << k << "\\n";
        return false;
      }
    } catch (...) {
      std::cerr << "invalid value for " << k << "\\n";
      return false;
    }
  }
  return true;
}

static void read_and_print(motorbridge::Motor& m, uint8_t rid, const char* name, uint32_t timeout_ms) {
  float v = m.get_register_f32(rid, timeout_ms);
  std::cout << name << " (rid=" << static_cast<int>(rid) << ") = " << v << "\\n";
}

static void write_if(motorbridge::Motor& m, uint8_t rid, const char* name, const std::optional<float>& v) {
  if (!v.has_value()) return;
  std::cout << "write " << name << " (rid=" << static_cast<int>(rid) << ") <= " << *v << "\\n";
  m.write_register_f32(rid, *v);
}

int main(int argc, char** argv) {
  Options o;
  if (!parse_args(argc, argv, o)) {
    if (argc <= 1 || std::string(argv[1]) != "--help") print_help();
    return (argc > 1 && std::string(argv[1]) == "--help") ? 0 : 2;
  }

  try {
    motorbridge::Controller ctrl(o.channel);
    auto m = ctrl.add_damiao_motor(o.motor_id, o.feedback_id, o.model);

    std::cout << "channel=" << o.channel << " model=" << o.model << " motor_id=0x" << std::hex
              << o.motor_id << " feedback_id=0x" << o.feedback_id << std::dec << "\\n";

    std::cout << "[before]\\n";
    read_and_print(m, 25, "KP_ASR", o.timeout_ms);
    read_and_print(m, 26, "KI_ASR", o.timeout_ms);
    read_and_print(m, 27, "KP_APR", o.timeout_ms);
    read_and_print(m, 28, "KI_APR", o.timeout_ms);
    read_and_print(m, 21, "PMAX", o.timeout_ms);
    read_and_print(m, 22, "VMAX", o.timeout_ms);
    read_and_print(m, 23, "TMAX", o.timeout_ms);
    read_and_print(m, 4, "ACC", o.timeout_ms);
    read_and_print(m, 5, "DEC", o.timeout_ms);
    read_and_print(m, 6, "MAX_SPD", o.timeout_ms);

    write_if(m, 25, "KP_ASR", o.kp_asr);
    write_if(m, 26, "KI_ASR", o.ki_asr);
    write_if(m, 27, "KP_APR", o.kp_apr);
    write_if(m, 28, "KI_APR", o.ki_apr);
    write_if(m, 21, "PMAX", o.pmax);
    write_if(m, 22, "VMAX", o.vmax);
    write_if(m, 23, "TMAX", o.tmax);
    write_if(m, 4, "ACC", o.acc);
    write_if(m, 5, "DEC", o.dec);
    write_if(m, 6, "MAX_SPD", o.max_spd);

    if (o.store) {
      m.store_parameters();
      std::cout << "store_parameters sent\\n";
    }

    std::cout << "[after]\\n";
    read_and_print(m, 25, "KP_ASR", o.timeout_ms);
    read_and_print(m, 26, "KI_ASR", o.timeout_ms);
    read_and_print(m, 27, "KP_APR", o.timeout_ms);
    read_and_print(m, 28, "KI_APR", o.timeout_ms);
    read_and_print(m, 21, "PMAX", o.timeout_ms);
    read_and_print(m, 22, "VMAX", o.timeout_ms);
    read_and_print(m, 23, "TMAX", o.timeout_ms);
    read_and_print(m, 4, "ACC", o.timeout_ms);
    read_and_print(m, 5, "DEC", o.timeout_ms);
    read_and_print(m, 6, "MAX_SPD", o.timeout_ms);

    return 0;
  } catch (const std::exception& e) {
    std::cerr << "error: " << e.what() << "\\n";
    return 1;
  }
}
