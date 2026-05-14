from ...ir.math_expr import (
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
    Variable,
    Vec3,
)
from ...ir.scene_nodes import BoxNode, SceneNode, SphereNode, UnionNode
from ...visitor.expr_visitor import ExprVisitor
from ...visitor.scene_visitor import SceneVisitor
from ..backends.base import Backend
from .wgsl_ast import (
    SdfResult,
    WGSLBinaryExpr,
    WGSLCallExpr,
    WGSLConstant,
    WGSLNode,
    WGSLVariable,
    WGSLVec3,
)


class WGSLExprCompiler(ExprVisitor):
    def visit_Constant(self, expr: Constant) -> WGSLNode:
        return WGSLConstant(value=expr.value)

    def visit_Variable(self, expr: Variable) -> WGSLNode:
        return WGSLVariable(name=expr.name)

    def visit_Add(self, expr: Add) -> WGSLNode:
        return WGSLBinaryExpr(
            op="+", left=self.visit(expr.left), right=self.visit(expr.right)
        )

    def visit_Sub(self, expr: Sub) -> WGSLNode:
        return WGSLBinaryExpr(
            op="-", left=self.visit(expr.left), right=self.visit(expr.right)
        )

    def visit_Mul(self, expr: Mul) -> WGSLNode:
        return WGSLBinaryExpr(
            op="*", left=self.visit(expr.left), right=self.visit(expr.right)
        )

    def visit_Div(self, expr: Div) -> WGSLNode:
        return WGSLBinaryExpr(
            op="/", left=self.visit(expr.left), right=self.visit(expr.right)
        )

    def visit_Max(self, expr: Max) -> WGSLNode:
        return WGSLCallExpr(
            func_name="max", args=[self.visit(expr.left), self.visit(expr.right)]
        )

    def visit_Min(self, expr: Min) -> WGSLNode:
        return WGSLCallExpr(
            func_name="min", args=[self.visit(expr.left), self.visit(expr.right)]
        )

    def visit_Pow(self, expr: Pow) -> WGSLNode:
        return WGSLCallExpr(
            func_name="pow", args=[self.visit(expr.left), self.visit(expr.right)]
        )

    def visit_Sin(self, expr: Sin) -> WGSLNode:
        return WGSLCallExpr(func_name="sin", args=[self.visit(expr.expr)])

    def visit_Cos(self, expr: Cos) -> WGSLNode:
        return WGSLCallExpr(func_name="cos", args=[self.visit(expr.expr)])

    def visit_Sqrt(self, expr: Sqrt) -> WGSLNode:
        return WGSLCallExpr(func_name="sqrt", args=[self.visit(expr.expr)])

    def visit_Vec3(self, expr: Vec3) -> WGSLNode:
        return WGSLVec3(
            x=self.visit(expr.x), y=self.visit(expr.y), z=self.visit(expr.z)
        )


class WGSLSceneCompiler(SceneVisitor):
    def __init__(self):
        self.expr_compiler = WGSLExprCompiler()

    def _get_material_id(self, node: SceneNode) -> WGSLNode:
        mat_id = node.material.id if node.material else 1
        return WGSLConstant(value=mat_id)

    def _transform_point(self, p_var: str, node: SceneNode) -> WGSLNode:
        # local space transform p - pos
        pos = node.transform.position
        pos_vec = WGSLVec3(
            x=WGSLConstant(value=pos[0]),
            y=WGSLConstant(value=pos[1]),
            z=WGSLConstant(value=pos[2]),
        )
        return WGSLBinaryExpr(op="-", left=WGSLVariable(name=p_var), right=pos_vec)

    def visit_SphereNode(self, node: SphereNode) -> WGSLNode:
        local_p = self._transform_point("p", node)
        radius = self.expr_compiler.visit(node.radius)
        dist = WGSLBinaryExpr(
            op="-", left=WGSLCallExpr(func_name="length", args=[local_p]), right=radius
        )
        return SdfResult(distance=dist, material_id=self._get_material_id(node))

    def visit_BoxNode(self, node: BoxNode) -> WGSLNode:
        local_p = self._transform_point("p", node)
        size = self.expr_compiler.visit(node.size)
        abs_p = WGSLCallExpr(func_name="abs", args=[local_p])
        diff = WGSLBinaryExpr(op="-", left=abs_p, right=size)

        zero_vec = WGSLVec3(x=WGSLConstant(0), y=WGSLConstant(0), z=WGSLConstant(0))
        max_val = WGSLCallExpr(func_name="max", args=[diff, zero_vec])

        dist = WGSLCallExpr(func_name="length", args=[max_val])
        return SdfResult(distance=dist, material_id=self._get_material_id(node))

    def visit_UnionNode(self, node: UnionNode) -> WGSLNode:
        left_res: SdfResult = self.visit(node.left)
        right_res: SdfResult = self.visit(node.right)

        # need a proper min that returns the vec2<f32>(dist, mat_id) with the smallest dist.
        #  WGSL AST doesn't easily inline a ternary operator without variable bindings,
        # we'll emit a custom function call like opU(left, right) which handles materials.
        # For simplicity here, we can assume opU exists in the runtime string.
        return WGSLCallExpr(func_name="opU", args=[left_res, right_res])


class WGSLBackend(Backend):
    def compile_expr(self, expr: Expr) -> WGSLNode:
        compiler = WGSLExprCompiler()
        return compiler.visit(expr)

    def compile_scene(self, node: SceneNode) -> WGSLNode:
        compiler = WGSLSceneCompiler()
        return compiler.visit(node)

    def emit_source(self, ast_node: WGSLNode) -> str:
        # Extra helper fn for union
        helpers = """
fn opU(d1: vec2<f32>, d2: vec2<f32>) -> vec2<f32> {
    if (d1.x < d2.x) { return d1; }
    return d2;
}
"""
        return helpers + "\\n// Generated Expression/Scene:\\n" + ast_node.emit()


# Legacy shims for compatibility with existing tests
def emit_expr(expr: Expr) -> str:
    backend = WGSLBackend()
    ast = backend.compile_expr(expr)
    return ast.emit()


def compile_node(node: SceneNode) -> str:
    backend = WGSLBackend()
    ast = backend.compile_scene(node)
    return ast.emit()
