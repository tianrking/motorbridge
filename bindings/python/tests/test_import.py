from motorbridge import Controller, Mode, MotorState


def test_import_symbols() -> None:
    assert Controller is not None
    assert Mode.MIT.value == 1
    assert MotorState is not None
