# Settings is a view, and navigation has a sidebar and a top bar

Settings is a page inside the workspace panel, not a dialog. The shell keeps its
sidebar and title bar while Settings is open, and the Settings screen is a rail
of destinations beside the open page: Preferences, Appearance, Notifications,
Providers, Projects, and Shortcuts. Providers is the existing coding-agent
screen; Notifications and Appearance hold desktop preferences that used to be
rows in one General page.

Navigation has two layouts, chosen in Settings › Preferences and stored in
`desktop-settings.json`: the destinations and chats down a resizable sidebar
(default), or the destinations as icon buttons in the title bar with the sidebar
hidden. A narrow window keeps the sidebar layout with its toggle, because the
top bar has no room for five buttons beside the chat tabs.

The layouts and row metrics come from the reference desktop app in `.or`
(`features/settings`, `shell/view/sidebar.rs`, `shell/view/titlebar.rs`). A
dialog like the earlier one could not carry six destinations, a rail, and page
headers without truncating the page, and it hid the workspace the settings
described. Making Settings a view lets a layout switch render live around the
page.

[Local settings reference](../rune/README.md)
