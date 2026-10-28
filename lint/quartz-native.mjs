import { definePlugin } from "vite-plus/lint/plugins";

const dimensions =
  /^(?:fontSize|lineHeight|fontWeight|fontFamily|letterSpacing|gap|padding(?:Top|Bottom|Left|Right)?|margin(?:Top|Bottom|Left|Right)?|border(?:Top|Bottom|Left|Right)?Width|borderRadius|outlineWidth|outlineOffset|minHeight|maxHeight|minWidth|maxWidth|height|width)$/;
const color = /(?:color|Color)$/;

export default definePlugin({
  meta: { name: "quartz-native", version: "1.0.0" },
  rules: {
    "controls-named": {
      meta: {
        type: "problem",
        schema: [],
        messages: {
          name: "Name this native control with aria-label or one Field label; grouped controls need separate aria-label values.",
        },
      },
      create(context) {
        const control = (node) =>
          ["Input", "Textarea"].includes(node.name?.name);
        const named = (node) =>
          node.attributes.some(
            (attribute) => attribute.name?.name === "aria-label"
          );
        function countControls(element) {
          let count = 0;
          function visit(node) {
            if (node.type === "JSXOpeningElement" && control(node)) count++;
            if (node.openingElement) visit(node.openingElement);
            for (const child of node.children ?? []) visit(child);
          }
          visit(element);
          return count;
        }
        return {
          JSXOpeningElement(node) {
            if (!control(node) || named(node)) return;
            const field = context.sourceCode
              .getAncestors(node)
              .findLast(
                (ancestor) =>
                  ancestor.type === "JSXElement" &&
                  ["Field", "Labeled"].includes(
                    ancestor.openingElement.name.name
                  )
              );
            if (!field || countControls(field) > 1)
              context.report({ node, messageId: "name" });
          },
        };
      },
    },
    "tokens-only": {
      meta: {
        type: "problem",
        schema: [],
        messages: {
          token:
            "Use a shared Quartz token for {{name}}; see @coderune/tokens/quartz.",
        },
      },
      create(context) {
        return {
          Property(node) {
            const owner = node.parent?.parent;
            if (
              owner?.type === "CallExpression" &&
              ["render", "createTestRoot"].includes(owner.callee.name) &&
              owner.arguments.includes(node.parent)
            )
              return;
            const name = node.key.name ?? node.key.value;
            if (!dimensions.test(name) && !color.test(name)) return;
            if (node.value.type !== "Literal") return;
            // Percentages and auto size the native viewport rather than an aesthetic role.
            if (
              typeof node.value.value === "string" &&
              /^(?:\d+%|auto)$/.test(node.value.value)
            )
              return;
            context.report({
              node: node.value,
              messageId: "token",
              data: { name },
            });
          },
        };
      },
    },
    "components-only": {
      meta: {
        type: "problem",
        schema: [],
        messages: {
          component:
            "Use native Quartz {{component}} from ./src/quartz instead of a raw host element.",
        },
      },
      create(context) {
        if (context.filename.endsWith("/src/quartz.tsx")) return {};
        const components = {
          div: "View",
          text: "Text",
          input: "Input",
          textarea: "Textarea",
        };
        return {
          JSXOpeningElement(node) {
            const component = components[node.name.name];
            if (component)
              context.report({
                node,
                messageId: "component",
                data: { component },
              });
          },
        };
      },
    },
  },
});
