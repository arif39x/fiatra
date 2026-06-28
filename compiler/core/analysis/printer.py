from ..ir.math_expr import BinaryOp, Constant, Expr, UnaryOp, Variable, Vec3
from ..visitor.expr_visitor import ExprVisitor


class TreePrinter(ExprVisitor):
    def __init__(self):
        self.indent = 0

    def _print_line(self, text: str, node: Expr):
        prefix = "  " * self.indent

        meta = []
        if node.value_type:
            meta.append(node.value_type.name)
        if node.span:
            meta.append(f"L{node.span.line}:C{node.span.column}")

        meta_str = f" [{', '.join(meta)}]" if meta else ""
        return f"{prefix}{text}{meta_str}\n"

    def visit_Constant(self, expr: Constant) -> str:
        return self._print_line(f"Constant({expr.value})", expr)

    def visit_Variable(self, expr: Variable) -> str:
        return self._print_line(f"Variable({expr.name})", expr)

    def visit_BinaryOp(self, expr: BinaryOp) -> str:
        res = self._print_line(f"{type(expr).__name__}", expr)
        self.indent += 1
        res += self.visit(expr.left)
        res += self.visit(expr.right)
        self.indent -= 1
        return res

    def visit_Add(self, expr: Expr) -> str:
        return self.visit_BinaryOp(expr)

    def visit_Sub(self, expr: Expr) -> str:
        return self.visit_BinaryOp(expr)

    def visit_Mul(self, expr: Expr) -> str:
        return self.visit_BinaryOp(expr)

    def visit_Div(self, expr: Expr) -> str:
        return self.visit_BinaryOp(expr)

    def visit_Pow(self, expr: Expr) -> str:
        return self.visit_BinaryOp(expr)

    def visit_Max(self, expr: Expr) -> str:
        return self.visit_BinaryOp(expr)

    def visit_Min(self, expr: Expr) -> str:
        return self.visit_BinaryOp(expr)

    def visit_UnaryOp(self, expr: UnaryOp) -> str:
        res = self._print_line(f"{type(expr).__name__}", expr)
        self.indent += 1
        res += self.visit(expr.expr)
        self.indent -= 1
        return res

    def visit_Sin(self, expr: Expr) -> str:
        return self.visit_UnaryOp(expr)

    def visit_Cos(self, expr: Expr) -> str:
        return self.visit_UnaryOp(expr)

    def visit_Sqrt(self, expr: Expr) -> str:
        return self.visit_UnaryOp(expr)

    def visit_Abs(self, expr: Expr) -> str:
        return self.visit_UnaryOp(expr)

    def visit_Vec3(self, expr: Vec3) -> str:
        res = self._print_line("Vec3", expr)
        self.indent += 1
        res += self.visit(expr.x)
        res += self.visit(expr.y)
        res += self.visit(expr.z)
        self.indent -= 1
        return res


def print_expr_tree(expr: Expr) -> str:
    """Returns a formatted string representing the expression tree."""
    printer = TreePrinter()
    return printer.visit(expr)
