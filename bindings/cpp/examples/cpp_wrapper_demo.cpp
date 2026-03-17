#include <chrono>
#include <iostream>
#include <thread>

#include "motorbridge/motorbridge.hpp"

int main() {
  try {
    motorbridge::Controller ctrl("can0");
    auto m = ctrl.add_damiao_motor(0x01, 0x11, "4340P");

    ctrl.enable_all();
    m.ensure_mode(motorbridge::Mode::MIT, 1000);

    for (int i = 0; i < 50; ++i) {
      m.send_mit(0.0f, 0.0f, 20.0f, 1.0f, 0.0f);
      auto st = m.get_state();
      if (st.has_value()) {
        std::cout << "#" << i << " pos=" << st->pos << " vel=" << st->vel << " torq=" << st->torq
                  << " status=" << static_cast<int>(st->status_code) << "\n";
      } else {
        std::cout << "#" << i << " no feedback yet\n";
      }
      std::this_thread::sleep_for(std::chrono::milliseconds(20));
    }

    ctrl.shutdown();
    return 0;
  } catch (const std::exception& e) {
    std::cerr << "error: " << e.what() << "\n";
    return 1;
  }
}

