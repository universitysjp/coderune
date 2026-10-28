import geistFont from "@coderune/tokens/fonts/geist-variable.ttf" with { type: "file" };
import paperMonoFont from "@coderune/tokens/fonts/paper-mono-variable.ttf" with { type: "file" };
import { quartz } from "@coderune/tokens/quartz";
import { Button as NativeButton, useGpuixRequired } from "@gpuix/react";
import type {
  ButtonProps as NativeButtonProps,
  InputProps,
  Props,
  StyleDesc,
  TextareaProps,
} from "@gpuix/react";
import {
  Children,
  createContext,
  isValidElement,
  useContext,
  useEffect,
  useState,
} from "react";
import type { ReactNode } from "react";

const {colors} = quartz;
const {space} = quartz;
const TextStyleContext = createContext<StyleDesc>({});
const FieldContext = createContext<{ label?: string; hint?: string }>({});

function controlName(children: ReactNode): string {
  return Children.toArray(children)
    .map((child) => {
      if (typeof child === "string" || typeof child === "number")
        {return String(child);}
      if (isValidElement<{ children?: ReactNode }>(child))
        {return controlName(child.props.children);}
      return "";
    })
    .join("")
    .trim();
}

const focusStyle: StyleDesc = {
  outlineWidth: quartz.line.focus,
  outlineColor: colors.ink,
  outlineOffset: quartz.line.focusOffset,
};

export function View({ style, ...props }: Props) {
  return <div {...props} style={{ minWidth: space[0], ...style }} />;
}

export interface TextProps extends Props {
  tone?: "ink" | "muted" | "faint" | "danger";
  mono?: boolean;
  size?: keyof typeof quartz.type;
}

export function Text({
  tone,
  mono,
  size,
  style,
  children,
  ...props
}: TextProps) {
  const inherited = useContext(TextStyleContext);
  const parts = Children.toArray(children);

  const content = parts.every(
    (part) => typeof part === "string" || typeof part === "number"
  )
    ? parts.join("")
    : children;

  return (
    <text
      {...props}
      style={{
        ...quartz.type.body,
        color: colors.ink,
        whiteSpace: "normal",
        ...inherited,
        ...(size ? quartz.type[size] : {}),
        ...(tone ? { color: colors[tone] } : {}),
        ...(mono ? { fontFamily: quartz.fonts.mono } : {}),
        ...style,
      }}
    >
      {content}
    </text>
  );
}

const fieldStyle: StyleDesc = {
  ...quartz.type.label,
  width: "100%",
  minWidth: space[0],
  minHeight: quartz.layout.control,
  paddingLeft: space[3],
  paddingRight: space[3],
  borderWidth: quartz.line.hair,
  borderColor: colors.controlLine,
  borderRadius: quartz.radius.soft,
  backgroundColor: colors.panel,
  color: colors.ink,
  hover: { borderColor: colors.controlHover },
  focusVisible: { ...focusStyle, borderColor: colors.ink },
};

const nativeTheme = {
  appearance: "light",
  bg: colors.panel,
  border: colors.controlLine,
  text: colors.ink,
  textMuted: colors.muted,
  textFaint: colors.faint,
  accent: colors.accent,
  caret: colors.ink,
  fontSans: quartz.fonts.sans,
  fontMono: quartz.fonts.mono,
} as const;

export function Input({ style, theme, ...props }: InputProps) {
  const field = useContext(FieldContext);

  return (
    <input
      
      aria-label={field.label}
      aria-description={field.hint}
      testId={`field-${(props["aria-label"] ?? field.label)?.toLowerCase().replaceAll(/[^a-z0-9]+/g, "-")}`}
      {...props}
      theme={{ ...nativeTheme, ...theme }}
      style={{ ...fieldStyle, ...style }}
    />
  );
}

export function Textarea({
  style,
  theme,
  minRows = 3,
  maxRows = 12,
  ...props
}: TextareaProps) {
  const field = useContext(FieldContext);

  return (
    <textarea
      
      aria-label={field.label}
      aria-description={field.hint}
      testId={`field-${(props["aria-label"] ?? field.label)?.toLowerCase().replaceAll(/[^a-z0-9]+/g, "-")}`}
      {...props}
      minRows={minRows}
      maxRows={Math.max(minRows, maxRows)}
      theme={{ ...nativeTheme, ...theme }}
      style={{
        ...fieldStyle,
        ...quartz.type.body,
        paddingTop: space[2],
        paddingBottom: space[2],
        ...style,
      }}
    />
  );
}

type ButtonVariant = "primary" | "secondary" | "ghost" | "destructive" | "link";

type ButtonSize = "xs" | "sm" | "default" | "lg";

const buttonHeights: Record<ButtonSize, number> = {
  xs: quartz.layout.controlXs,
  sm: quartz.layout.controlSm,
  default: quartz.layout.control,
  lg: quartz.layout.controlLg,
};

export interface ButtonProps extends NativeButtonProps {
  variant?: ButtonVariant;
  size?: ButtonSize;
  pending?: boolean;
  pendingLabel?: string;
  pressed?: boolean;
}

export function Button({
  variant = "secondary",
  size = "default",
  pending = false,
  pendingLabel,
  pressed = false,
  disabled = false,
  style,
  children,
  ...props
}: ButtonProps) {
  const primary = variant === "primary";
  const destructive = variant === "destructive";
  const quiet = variant === "ghost" || variant === "link";

  const textColor = primary
    ? colors.inverse
    : destructive
      ? colors.danger
      : colors.ink;

  const type = size === "lg" ? quartz.type.button : quartz.type.label;

  return (
    <NativeButton
      aria-label={controlName(children)}
      {...props}
      disabled={disabled || pending}
      aria-selected={pressed || undefined}
      aria-description={pending ? pendingLabel : props["aria-description"]}
      style={(state) => ({
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        flexShrink: space[0],
        minHeight: buttonHeights[size],
        maxWidth: "100%",
        paddingLeft: space[size === "xs" ? 2 : 3],
        paddingRight: space[size === "xs" ? 2 : 3],
        paddingTop: space[1],
        paddingBottom: space[1],
        borderWidth: quartz.line.hair,
        borderRadius: quartz.radius.soft,
        borderColor: destructive
          ? colors.danger
          : quiet
            ? colors.transparent
            : colors.controlLine,
        backgroundColor: primary
          ? colors.ink
          : pressed
            ? colors.accentSoft
            : quiet
              ? colors.transparent
              : colors.panel,
        color: textColor,
        cursor: state.disabled ? "not-allowed" : "pointer",
        opacity: state.disabled ? quartz.opacity.disabled : 1,
        hover: state.disabled
          ? undefined
          : {
              backgroundColor: primary
                ? colors.primaryHover
                : pressed
                  ? colors.accentSoft
                  : destructive
                    ? colors.dangerSoft
                    : colors.hover,
              borderColor: destructive
                ? colors.danger
                : quiet
                  ? colors.transparent
                  : colors.controlHover,
            },
        active: state.disabled
          ? undefined
          : {
              backgroundColor: primary
                ? colors.primaryHover
                : pressed
                  ? colors.accentSoft
                  : colors.pressed,
            },
        focusVisible: focusStyle,
        ...(typeof style === "function" ? style(state) : style),
      })}
    >
      <TextStyleContext.Provider
        value={{ ...type, color: textColor, whiteSpace: "normal" }}
      >
        <View
          style={{
            position: "relative",
            maxWidth: "100%",
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
          }}
        >
          <View style={{ opacity: pending && pendingLabel ? space[0] : 1 }}>
            {typeof children === "string" || typeof children === "number" ? (
              <Text>{children}</Text>
            ) : (
              children
            )}
          </View>
          {pending && pendingLabel ? (
            <View
              style={{
                position: "absolute",
                left: space[0],
                top: space[0],
                width: "100%",
                height: "100%",
                alignItems: "center",
                justifyContent: "center",
              }}
            >
              <Text>{pendingLabel}</Text>
            </View>
          ) : null}
        </View>
      </TextStyleContext.Provider>
    </NativeButton>
  );
}

export interface CardProps extends Props {
  title?: string;
  description?: string;
}

export function Card({
  title,
  description,
  children,
  style,
  ...props
}: CardProps) {
  return (
    <View
      role="group"
      aria-label={title}
      {...props}
      style={{
        display: "flex",
        flexDirection: "column",
        borderWidth: quartz.line.hair,
        borderColor: colors.line,
        borderRadius: quartz.radius.frame,
        backgroundColor: colors.panel,
        ...style,
      }}
    >
      {title ? (
        <View
          style={{
            display: "flex",
            flexDirection: "column",
            gap: space[1],
            padding: space[4],
            borderBottomWidth: quartz.line.hair,
            borderColor: colors.rowLine,
          }}
        >
          <Text size="headingXs" role="heading" aria-level={2}>
            {title}
          </Text>
          {description ? (
            <Text size="bodySm" tone="muted">
              {description}
            </Text>
          ) : null}
        </View>
      ) : null}
      <View
        style={{
          display: "flex",
          flexDirection: "column",
          gap: space[4],
          padding: space[4],
        }}
      >
        {children}
      </View>
    </View>
  );
}

export interface FieldProps extends Props {
  label: string;
  hint?: string;
}

export function Field({ label, hint, children, style, ...props }: FieldProps) {
  return (
    <FieldContext.Provider value={{ label, hint }}>
      <View
        role="group"
        aria-label={label}
        {...props}
        style={{
          display: "flex",
          flexDirection: "column",
          gap: space[2],
          ...style,
        }}
      >
        <Text size="label">{label}</Text>
        {children}
        {hint ? (
          <Text size="bodySm" tone="muted">
            {hint}
          </Text>
        ) : null}
      </View>
    </FieldContext.Provider>
  );
}

export interface AlertProps extends Props {
  error?: string;
  message?: string;
  announce?: boolean;
}

export function Alert({
  error,
  message,
  announce = true,
  children,
  style,
  ...props
}: AlertProps) {
  if (!error && !message && !children) {return null;}
  return (
    <View
      role={announce ? (error ? "alert" : "status") : "note"}
      {...props}
      style={{
        display: "flex",
        flexDirection: "column",
        gap: space[1],
        padding: space[3],
        borderWidth: quartz.line.hair,
        borderColor: colors.line,
        borderRadius: quartz.radius.soft,
        backgroundColor: colors.panel,
        ...style,
      }}
    >
      {error ? (
        <Text size="label" tone="danger">
          Action failed
        </Text>
      ) : null}
      <Text size="bodySm" tone="muted">
        {error || message || children}
      </Text>
    </View>
  );
}

export interface PageHeaderProps extends Props {
  title: string;
  description?: string;
  actions?: ReactNode;
}

export function PageHeader({
  title,
  description,
  actions,
  children,
  style,
  ...props
}: PageHeaderProps) {
  return (
    <View
      {...props}
      style={{
        display: "flex",
        flexDirection: "row",
        flexWrap: "wrap",
        alignItems: "center",
        gap: space[4],
        ...style,
      }}
    >
      <View
        style={{
          display: "flex",
          flexDirection: "column",
          gap: space[2],
          flexGrow: 1,
          maxWidth: quartz.layout.pageHeaderMax,
        }}
      >
        <Text size="headingLg" role="heading" aria-level={1}>
          {title}
        </Text>
        {description ? <Text tone="muted">{description}</Text> : null}
        {children}
      </View>
      {actions ? (
        <View
          style={{
            display: "flex",
            flexDirection: "row",
            flexWrap: "wrap",
            gap: space[2],
          }}
        >
          {actions}
        </View>
      ) : null}
    </View>
  );
}

export function Sidebar({ children, style, ...props }: Props) {
  return (
    <View
      role="navigation"
      aria-label="Workspace"
      {...props}
      style={{
        display: "flex",
        flexDirection: "column",
        width: quartz.layout.sidebar,
        flexShrink: space[0],
        height: "100%",
        padding: space[4],
        gap: space[1],
        borderRightWidth: quartz.line.hair,
        borderColor: colors.line,
        backgroundColor: colors.panel,
        ...style,
      }}
    >
      {children}
    </View>
  );
}

export function Badge({ children, style, ...props }: Props) {
  if (children === null || children === undefined || children === "")
    {return null;}
  return (
    <View
      {...props}
      style={{
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        minHeight: quartz.layout.tag,
        paddingLeft: space[1],
        paddingRight: space[1],
        borderRadius: quartz.radius.inner,
        backgroundColor: colors.code,
        ...style,
      }}
    >
      <Text size="labelSm">{children}</Text>
    </View>
  );
}

export async function registerQuartzFonts(renderer: {
  registerFont(bytes: Buffer): void;
}) {
  for (const path of [geistFont, paperMonoFont]) {
    renderer.registerFont(Buffer.from(await Bun.file(path).arrayBuffer()));
  }
}

export function QuartzFonts({ children }: { children: ReactNode }) {
  const renderer = useGpuixRequired();
  const [ready, setReady] = useState(false);
  const [error, setError] = useState<Error | null>(null);

  useEffect(() => {
    let mounted = true;

    const load = async () => {
      if (!renderer.registerFont) {
        throw new Error("GPUIX renderer does not support font registration");
      }

      await registerQuartzFonts({
        registerFont: renderer.registerFont.bind(renderer),
      });

      if (mounted) {setReady(true);}
    };

    void load().catch((error) => {
      if (mounted)
        setError(error instanceof Error ? error : new Error(String(error)));
    });

    return () => {
      mounted = false;
    };
  }, [renderer]);

  if (error) {throw error;}
  return ready ? children : null;
}
