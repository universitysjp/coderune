// Phosphor is the requested icon family for the native workspace shell.
// Bun's direct SVG re-export treats the asset as a path instead of text.
/* oxlint-disable quartz/no-second-icon-set, unicorn/prefer-export-from */
import documentsIcon from "@phosphor-icons/core/regular/files.svg" with { type: "text" };
import folderIcon from "@phosphor-icons/core/regular/folder-simple.svg" with { type: "text" };
import gearIcon from "@phosphor-icons/core/regular/gear.svg" with { type: "text" };
import houseIcon from "@phosphor-icons/core/regular/house.svg" with { type: "text" };
import agentsIcon from "@phosphor-icons/core/regular/robot.svg" with { type: "text" };
import captureIcon from "@phosphor-icons/core/regular/scan.svg" with { type: "text" };
import sidebarIcon from "@phosphor-icons/core/regular/sidebar-simple.svg" with { type: "text" };
import userIcon from "@phosphor-icons/core/regular/user-circle.svg" with { type: "text" };
import closeIcon from "@phosphor-icons/core/regular/x.svg" with { type: "text" };

export {
  agentsIcon,
  captureIcon,
  closeIcon,
  documentsIcon,
  folderIcon,
  gearIcon,
  houseIcon,
  sidebarIcon,
  userIcon,
};
