from ..errors import ValidationError
from ..ir.math_expr import (
    Abs,
    Add,
    BinaryOp,
    Constant,
    Cos,
    Div,
    Expr,
    Max,
    Min,
    Mul,
    Pow,
    Sin,
    Sqrt,
    Sub,
    UnaryOp,
    Variable,
    Vec3,
)
from ..visitor.expr_visitor import ExprVisitor


class ValidatorPass(ExprVisitor):
    def visit_Constant(self, expr: Constant):
        pass

    def visit_Variable(self, expr: Variable):
        if expr.name not in {
            "x",
            "y",
            "z",
            "state.x",
            "state.y",
            "state.z",
            "state_x",
            "state_y",
            "state_z",
        }:
            raise ValidationError(
                f"Invalid variable name: {expr.name}. Only x, y, z, and state variables are allowed.",
                span=expr.span,
                node_type="Variable",
            )

    def visit_Div(self, expr: Div):
        self.visit(expr.left)
        self.visit(expr.right)
        if isinstance(expr.right, Constant) and expr.right.value == 0:
            raise ValidationError(
                "Division by constant zero detected in IR.",
                span=expr.span,
                node_type="Div",
            )

    def visit_Pow(self, expr: Pow):
        self.visit(expr.left)
        self.visit(expr.right)
        if isinstance(expr.right, Constant) and expr.right.value > 8:
            raise ValidationError(
                f"Power exponent {expr.right.value} exceeds maximum allowed value of 8.",
                span=expr.span,
                node_type="Pow",
            )

    def visit_BinaryOp(self, expr: BinaryOp):
        self.visit(expr.left)
        self.visit(expr.right)

    def visit_Add(self, expr: Add):
        self.visit_BinaryOp(expr)

    def visit_Sub(self, expr: Sub):
        self.visit_BinaryOp(expr)

    def visit_Mul(self, expr: Mul):
        self.visit_BinaryOp(expr)

    def visit_Max(self, expr: Max):
        self.visit_BinaryOp(expr)

    def visit_Min(self, expr: Min):
        self.visit_BinaryOp(expr)

    def visit_UnaryOp(self, expr: UnaryOp):
        self.visit(expr.expr)

    def visit_Sin(self, expr: Sin):
        self.visit_UnaryOp(expr)

    def visit_Cos(self, expr: Cos):
        self.visit_UnaryOp(expr)

    def visit_Sqrt(self, expr: Sqrt):
        self.visit_UnaryOp(expr)

    def visit_Abs(self, expr: Abs):
        self.visit_UnaryOp(expr)

    def visit_Vec3(self, expr: Vec3):
        self.visit(expr.x)
        self.visit(expr.y)
        self.visit(expr.z)


def validate_expr(expr: Expr):

    visitor = ValidatorPass()
    visitor.visit(expr)
