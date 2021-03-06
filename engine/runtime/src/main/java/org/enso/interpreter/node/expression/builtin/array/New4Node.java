package org.enso.interpreter.node.expression.builtin.array;

import com.oracle.truffle.api.nodes.Node;
import org.enso.interpreter.dsl.BuiltinMethod;
import org.enso.interpreter.runtime.data.Array;

@BuiltinMethod(
    type = "Array",
    name = "new_4",
    description = "Creates an array with four given elements.")
public class New4Node extends Node {

  Object execute(Object _this, Object item_1, Object item_2, Object item_3, Object item_4) {
    return new Array(item_1, item_2, item_3, item_4);
  }
}
