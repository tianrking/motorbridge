class MotorBridgeError(RuntimeError):
    """Base error for motorbridge Python SDK."""


class AbiLoadError(MotorBridgeError):
    """Raised when libmotor_abi cannot be found or loaded."""


class CallError(MotorBridgeError):
    """Raised when ABI call returns non-zero status."""
