from dataclasses import dataclass

from ..errors import ComplexityError
from ..ir.math_expr import BinaryOp, Expr, Pow, UnaryOp, Vec3, Constant
from ..visitor.expr_visitor import ExprVisitor


@dataclass
class ComplexityConfig:
    max_nodes: int = 1000
    max_depth: int = 50
    max_pow_exponent: float = 8.0


class ComplexityAnalyzer(ExprVisitor):
    def __init__(self, config: ComplexityConfig):
        self.config = config
        self.node_count = 0
        self.current_depth = 0

    def _check_limits(self, expr: Expr):
        self.node_count += 1
        if self.node_count > self.config.max_nodes:
            raise ComplexityError(
                f"Expression exceeds max nodes ({self.config.max_nodes})",
                span=expr.span,
            )
        if self.current_depth > self.config.max_depth:
            raise ComplexityError(
                f"Expression exceeds max depth ({self.config.max_depth})",
                span=expr.span,
            )

    def generic_visit(self, expr: Expr, *args, **kwargs):
        self._check_limits(expr)

    def visit_Pow(self, expr: Pow):
        self._check_limits(expr)

        if (
            isinstance(expr.right, Constant)
            and expr.right.value > self.config.max_pow_exponent
        ):
            raise ComplexityError(
                f"Power exponent {expr.right.value} exceeds max allowed ({self.config.max_pow_exponent})",
                span=expr.span,
                node_type="Pow",
            )

        self.current_depth += 1
        self.visit(expr.left)
        self.visit(expr.right)
        self.current_depth -= 1

    def visit_BinaryOp(self, expr: BinaryOp):
        self._check_limits(expr)
        self.current_depth += 1
        self.visit(expr.left)
        self.visit(expr.right)
        self.current_depth -= 1

    def visit_Add(self, expr: Expr):
        self.visit_BinaryOp(expr)

    def visit_Sub(self, expr: Expr):
        self.visit_BinaryOp(expr)

    def visit_Mul(self, expr: Expr):
        self.visit_BinaryOp(expr)

    def visit_Div(self, expr: Expr):
        self.visit_BinaryOp(expr)

    def visit_Max(self, expr: Expr):
        self.visit_BinaryOp(expr)

    def visit_Min(self, expr: Expr):
        self.visit_BinaryOp(expr)

    def visit_UnaryOp(self, expr: UnaryOp):
        self._check_limits(expr)
        self.current_depth += 1
        self.visit(expr.expr)
        self.current_depth -= 1

    def visit_Sin(self, expr: Expr):
        self.visit_UnaryOp(expr)

    def visit_Cos(self, expr: Expr):
        self.visit_UnaryOp(expr)

    def visit_Sqrt(self, expr: Expr):
        self.visit_UnaryOp(expr)

    def visit_Abs(self, expr: Expr):
        self.visit_UnaryOp(expr)

    def visit_Vec3(self, expr: Vec3):
        self._check_limits(expr)
        self.current_depth += 1
        self.visit(expr.x)
        self.visit(expr.y)
        self.visit(expr.z)
        self.current_depth -= 1


def analyze_complexity(
    expr: Expr, config: ComplexityConfig = ComplexityConfig()
) -> Expr:
    visitor = ComplexityAnalyzer(config)
    visitor.visit(expr)
    object.__setattr__(expr, "complexity_score", visitor.node_count)
    return expr
