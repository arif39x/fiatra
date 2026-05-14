from dataclasses import dataclass, field
from typing import Optional
from .types import ValueType
from ..errors import SourceSpan

@dataclass(frozen=True, kw_only=True)
class Expr:
    """Base class for all mathematical expression nodes."""
    span: Optional[SourceSpan] = field(default=None, compare=False, hash=False)
    value_type: Optional[ValueType] = field(default=None, compare=False, hash=False)

@dataclass(frozen=True, kw_only=True)
class Constant(Expr):
    value: float

@dataclass(frozen=True, kw_only=True)
class Variable(Expr):
    name: str

@dataclass(frozen=True, kw_only=True)
class BinaryOp(Expr):
    left: Expr
    right: Expr

@dataclass(frozen=True, kw_only=True)
class Add(BinaryOp):
    pass

@dataclass(frozen=True, kw_only=True)
class Sub(BinaryOp):
    pass

@dataclass(frozen=True, kw_only=True)
class Mul(BinaryOp):
    pass

@dataclass(frozen=True, kw_only=True)
class Div(BinaryOp):
    pass

@dataclass(frozen=True, kw_only=True)
class Pow(BinaryOp):
    pass

@dataclass(frozen=True, kw_only=True)
class UnaryOp(Expr):
    expr: Expr

@dataclass(frozen=True, kw_only=True)
class Sin(UnaryOp):
    pass

@dataclass(frozen=True, kw_only=True)
class Cos(UnaryOp):
    pass

@dataclass(frozen=True, kw_only=True)
class Sqrt(UnaryOp):
    pass

@dataclass(frozen=True, kw_only=True)
class Max(BinaryOp):
    pass

@dataclass(frozen=True, kw_only=True)
class Min(BinaryOp):
    pass

@dataclass(frozen=True, kw_only=True)
class Vec3(Expr):
    x: Expr
    y: Expr
    z: Expr
