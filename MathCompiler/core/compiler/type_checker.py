from ..ir.types import ValueType
from ..ir.math_expr import (
    Expr, Constant, Variable, Add, Sub, Mul, Div, Pow, Sin, Cos, Sqrt, Vec3, BinaryOp, UnaryOp
)
from ..visitor.expr_visitor import ExprTransformer
from ..errors import TypeError

class TypeInferencer(ExprTransformer):
    """Infers and validates types, returning new nodes with value_type populated."""

    def visit_Constant(self, expr: Constant) -> Expr:
        return Constant(value=expr.value, span=expr.span, value_type=ValueType.SCALAR)

    def visit_Variable(self, expr: Variable) -> Expr:
        # Currently, all variables in equations (x, y, z) are scalars.
        # Future symbol table lookup goes here.
        return Variable(name=expr.name, span=expr.span, value_type=ValueType.SCALAR)

    def visit_Vec3(self, expr: Vec3) -> Expr:
        x = self.visit(expr.x)
        y = self.visit(expr.y)
        z = self.visit(expr.z)
        
        for c, name in zip([x, y, z], ['x', 'y', 'z']):
            if c.value_type != ValueType.SCALAR:
                raise TypeError(f"Vec3 component {name} must be SCALAR, got {c.value_type}", span=expr.span, node_type="Vec3")
                
        return Vec3(x=x, y=y, z=z, span=expr.span, value_type=ValueType.VEC3)

    def _visit_binary_op(self, expr: BinaryOp, node_class: type) -> Expr:
        left = self.visit(expr.left)
        right = self.visit(expr.right)

        if left.value_type != right.value_type:
            # Allow scalar multiplication with vectors, etc. (Not fully implemented here, keeping it strict for now unless Mul)
            if node_class == Mul and ((left.value_type == ValueType.SCALAR and right.value_type == ValueType.VEC3) or 
                                      (left.value_type == ValueType.VEC3 and right.value_type == ValueType.SCALAR)):
                return node_class(left=left, right=right, span=expr.span, value_type=ValueType.VEC3)
                
            raise TypeError(f"Cannot perform {node_class.__name__} on {left.value_type} and {right.value_type}", span=expr.span, node_type=node_class.__name__)

        # If they are same, return same type
        return node_class(left=left, right=right, span=expr.span, value_type=left.value_type)

    def visit_Add(self, expr: Add) -> Expr:
        return self._visit_binary_op(expr, Add)

    def visit_Sub(self, expr: Sub) -> Expr:
        return self._visit_binary_op(expr, Sub)

    def visit_Mul(self, expr: Mul) -> Expr:
        return self._visit_binary_op(expr, Mul)

    def visit_Div(self, expr: Div) -> Expr:
        return self._visit_binary_op(expr, Div)

    def visit_Max(self, expr: Max) -> Expr:
        return self._visit_binary_op(expr, Max)

    def visit_Min(self, expr: Min) -> Expr:
        return self._visit_binary_op(expr, Min)

    def visit_Pow(self, expr: Pow) -> Expr:
        left = self.visit(expr.left)
        right = self.visit(expr.right)
        
        if left.value_type != ValueType.SCALAR or right.value_type != ValueType.SCALAR:
            raise TypeError(f"Pow requires SCALAR operands, got {left.value_type} and {right.value_type}", span=expr.span, node_type="Pow")
            
        return Pow(left=left, right=right, span=expr.span, value_type=ValueType.SCALAR)

    def _visit_unary_op(self, expr: UnaryOp, node_class: type) -> Expr:
        inner = self.visit(expr.expr)
        if inner.value_type != ValueType.SCALAR:
            raise TypeError(f"{node_class.__name__}() requires SCALAR input, got {inner.value_type}", span=expr.span, node_type=node_class.__name__)
        return node_class(expr=inner, span=expr.span, value_type=ValueType.SCALAR)

    def visit_Sin(self, expr: Sin) -> Expr:
        return self._visit_unary_op(expr, Sin)

    def visit_Cos(self, expr: Cos) -> Expr:
        return self._visit_unary_op(expr, Cos)

    def visit_Sqrt(self, expr: Sqrt) -> Expr:
        return self._visit_unary_op(expr, Sqrt)

def infer_type(expr: Expr) -> Expr:
    """Infers types for an expression tree and returns a fully typed tree."""
    visitor = TypeInferencer()
    return visitor.visit(expr)

def validate_types(expr: Expr) -> None:
    """Validates that a tree has correct types. Returns nothing or throws TypeError."""
    # Since TypeInferencer also validates, we can just run it.
    infer_type(expr)
