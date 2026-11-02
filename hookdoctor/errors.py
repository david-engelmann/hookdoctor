"""Errors raised by hookdoctor."""


class HookdoctorError(Exception):
    """Raised when an input or operation cannot be trusted."""


class ReportValidationError(HookdoctorError):
    """Raised when a report disagrees with a rebuilt examination."""
