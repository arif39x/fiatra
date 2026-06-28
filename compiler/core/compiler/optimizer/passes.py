import math

from ...ir.math_expr import (
    Abs,
    Add,
    Constant,
    Cos,
    Div,
    Expr,
    Mul,
    Pow,
    Sin,
    Sqrt,
    Sub,
    Vec3,
)
from ...visitor.expr_visitor import ExprTransformer


class OptimizerPass(ExprTransformer):
    def visit_Add(self, expr: Add) -> Expr:
        left = self.visit(expr.left)
        right = self.visit(expr.right)

        # Constant Folding
        if isinstance(left, Constant) and isinstance(right, Constant):
            return Constant(
                value=left.value + right.value,
                span=expr.span,
                value_type=expr.value_type,
            )

        # Algebraic Simplification
        if isinstance(left, Constant) and left.value == 0.0:
            return right
        if isinstance(right, Constant) and right.value == 0.0:
            return left

        return Add(left=left, right=right, span=expr.span, value_type=expr.value_type)

    def visit_Sub(self, expr: Sub) -> Expr:
        left = self.visit(expr.left)
        right = self.visit(expr.right)

        if isinstance(left, Constant) and isinstance(right, Constant):
            return Constant(
                value=left.value - right.value,
                span=expr.span,
                value_type=expr.value_type,
            )

        if isinstance(right, Constant) and right.value == 0.0:
            return left

        return Sub(left=left, right=right, span=expr.span, value_type=expr.value_type)

    def visit_Mul(self, expr: Mul) -> Expr:
        left = self.visit(expr.left)
        right = self.visit(expr.right)

        if isinstance(left, Constant) and isinstance(right, Constant):
            return Constant(
                value=left.value * right.value,
                span=expr.span,
                value_type=expr.value_type,
            )

        if isinstance(left, Constant):
            if left.value == 1.0:
                return right
            if left.value == 0.0:
                return Constant(value=0.0, span=expr.span, value_type=expr.value_type)

        if isinstance(right, Constant):
            if right.value == 1.0:
                return left
            if right.value == 0.0:
                return Constant(value=0.0, span=expr.span, value_type=expr.value_type)

        return Mul(left=left, right=right, span=expr.span, value_type=expr.value_type)

    def visit_Div(self, expr: Div) -> Expr:
        left = self.visit(expr.left)
        right = self.visit(expr.right)

        if (
            isinstance(left, Constant)
            and isinstance(right, Constant)
            and right.value != 0
        ):
            return Constant(
                value=left.value / right.value,
                span=expr.span,
                value_type=expr.value_type,
            )

        return Div(left=left, right=right, span=expr.span, value_type=expr.value_type)

    def visit_Pow(self, expr: Pow) -> Expr:
        left = self.visit(expr.left)
        right = self.visit(expr.right)

        if isinstance(left, Constant) and isinstance(right, Constant):
            return Constant(
                value=left.value**right.value,
                span=expr.span,
                value_type=expr.value_type,
            )

        return Pow(left=left, right=right, span=expr.span, value_type=expr.value_type)

    def visit_Sin(self, expr: Sin) -> Expr:
        inner = self.visit(expr.expr)
        if isinstance(inner, Constant):
            return Constant(
                value=math.sin(inner.value), span=expr.span, value_type=expr.value_type
            )
        return Sin(expr=inner, span=expr.span, value_type=expr.value_type)

    def visit_Cos(self, expr: Cos) -> Expr:
        inner = self.visit(expr.expr)
        if isinstance(inner, Constant):
            return Constant(
                value=math.cos(inner.value), span=expr.span, value_type=expr.value_type
            )
        return Cos(expr=inner, span=expr.span, value_type=expr.value_type)

    def visit_Sqrt(self, expr: Sqrt) -> Expr:
        inner = self.visit(expr.expr)
        if isinstance(inner, Constant) and inner.value >= 0:
            return Constant(
                value=math.sqrt(inner.value), span=expr.span, value_type=expr.value_type
            )
        return Sqrt(expr=inner, span=expr.span, value_type=expr.value_type)

    def visit_Abs(self, expr: Abs) -> Expr:
        inner = self.visit(expr.expr)
        if isinstance(inner, Constant):
            return Constant(
                value=abs(inner.value), span=expr.span, value_type=expr.value_type
            )
        return Abs(expr=inner, span=expr.span, value_type=expr.value_type)


def optimize(expr: Expr) -> Expr:
    """Applies optimization passes to the Math IR tree."""
    visitor = OptimizerPass()
    return visitor.visit(expr)
