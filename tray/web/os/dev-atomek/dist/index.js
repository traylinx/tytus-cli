const __tytusWorkbenchCss="@import\"https://fonts.googleapis.com/css2?family=Archivo+Black&family=Space+Grotesk:wght@400;500;600;700&family=JetBrains+Mono:wght@400;500;700&family=Permanent+Marker&display=swap\";:root{--atomek-void: #0F1012;--atomek-asphalt: #1C1D20;--atomek-concrete: #3A3B3F;--atomek-steel: #6B6B6B;--atomek-chalk: #E5E0D2;--atomek-bone: #F0EAD8;--atomek-acid: #C4FF00;--atomek-fission: #FF2D87;--atomek-hazard: #FF6A1A;--atomek-cobalt: #2438FF;--atomek-font-display: \"Archivo Black\", Impact, sans-serif;--atomek-font-body: \"Space Grotesk\", system-ui, -apple-system, BlinkMacSystemFont, \"Segoe UI\", sans-serif;--atomek-font-mono: \"JetBrains Mono\", \"SF Mono\", Consolas, monospace;--workbench-bg: var(--atomek-void);--workbench-side: #111214;--workbench-panel: var(--atomek-asphalt);--workbench-panel-2: #151619;--workbench-border: #24262a;--workbench-border-strong: var(--atomek-concrete);--workbench-text: var(--atomek-bone);--workbench-muted: #9b978a;--workbench-blue: var(--atomek-acid);--workbench-blue-2: var(--atomek-acid);--workbench-purple: var(--atomek-acid);--workbench-input: #151619}.workbench-workbench{width:100%;height:100%;min-height:0;display:grid;grid-template-columns:48px minmax(0,var(--workbench-primary-width, 300px)) minmax(360px,1fr) minmax(0,var(--workbench-secondary-width, 520px));grid-template-rows:minmax(0,1fr) 22px;color:var(--workbench-text);background:var(--workbench-bg);font-family:-apple-system,BlinkMacSystemFont,Segoe UI,sans-serif;font-size:13px;overflow:hidden}.workbench-workbench.no-primary{grid-template-columns:48px 0 minmax(360px,1fr) minmax(0,var(--workbench-secondary-width, 430px))}.workbench-workbench.no-secondary{grid-template-columns:48px minmax(0,var(--workbench-primary-width, 300px)) minmax(360px,1fr) 0}.workbench-workbench.no-primary .workbench-statusbar{grid-column:3 / 5}.workbench-workbench.no-primary.no-secondary .workbench-statusbar{grid-column:3 / 4}button{font:inherit}.workbench-activity-bar{grid-row:1 / 3;background:#181818;border-right:1px solid var(--workbench-border);display:flex;flex-direction:column;align-items:center;padding:8px 0;gap:4px}.workbench-activity-spacer{flex:1}.workbench-activity-button{width:48px;height:46px;border:0;background:transparent;color:#858585;display:grid;place-items:center;cursor:pointer;position:relative}.workbench-activity-button:hover,.workbench-activity-button.active{color:#fff}.workbench-activity-button.active:before{content:\"\";position:absolute;left:0;top:6px;bottom:6px;width:2px;background:var(--workbench-blue)}.workbench-primary-region{grid-column:2 / 3;grid-row:1 / 2;min-width:0;min-height:0;position:relative;display:grid}.workbench-primary-region .workbench-sidebar{height:100%}.workbench-primary-resizer{position:absolute;right:-4px;top:0;bottom:0;width:7px;cursor:col-resize;z-index:5}.workbench-primary-resizer:hover{background:var(--workbench-blue)}.workbench-sidebar{background:var(--workbench-panel);border-right:1px solid var(--workbench-border);min-width:0;overflow:hidden;display:flex;flex-direction:column}.workbench-sidebar-title{height:35px;display:flex;align-items:center;padding:0 16px;font-size:11px;font-weight:700;letter-spacing:.08em;text-transform:uppercase;color:#bbb}.workbench-sidebar-scroll{overflow:auto;padding:0 12px 16px}.workbench-section-title{margin:12px 0 6px;color:#bbb;font-size:11px;font-weight:700;letter-spacing:.06em;text-transform:uppercase}.workbench-muted{color:var(--workbench-muted)}.workbench-button-blue{width:100%;height:31px;border:0;border-radius:2px;background:#0e639c;color:#fff;cursor:pointer;margin:4px 0}.workbench-button-blue:hover{background:#17b}.workbench-button-subtle{height:28px;border:1px solid var(--workbench-border-strong);border-radius:2px;background:#2d2d2d;color:var(--workbench-text);cursor:pointer;display:inline-flex;align-items:center;justify-content:center;gap:6px;padding:0 10px}.workbench-button-subtle:hover{background:#373737}.workbench-button-subtle.full{width:100%;margin:4px 0}.workbench-sidebar-actions{display:grid;grid-template-columns:1fr 1fr;gap:5px;margin:6px 0 8px}.workbench-input{width:100%;height:30px;border:1px solid var(--workbench-border-strong);border-radius:2px;background:var(--workbench-input);color:var(--workbench-text);padding:0 9px;outline:none;box-sizing:border-box}.workbench-input:focus{border-color:var(--workbench-blue)}.workbench-file-row,.workbench-tree-row,.workbench-folder-row{width:100%;min-height:24px;border:0;border-radius:2px;background:transparent;color:var(--workbench-text);display:flex;align-items:center;gap:6px;padding:3px 6px 3px calc(6px + (var(--workbench-depth, 0) * 12px));text-align:left;cursor:pointer}.workbench-folder-row{width:100%;border:0;background:transparent;cursor:pointer;color:#c9c9c9;font-weight:550}.workbench-file-row:hover,.workbench-tree-row:hover,.workbench-folder-row:hover{background:#2a2d2e}.workbench-file-row.active,.workbench-tree-row.active{background:#37373d;color:#fff}.workbench-row-name{overflow:hidden;text-overflow:ellipsis;white-space:nowrap}.workbench-row-meta{margin-left:auto;color:var(--workbench-muted);font-size:11px}.workbench-chevron-collapsed{transform:rotate(-90deg)}.workbench-search-group{margin:4px 0 10px}.workbench-search-hit{width:calc(100% - 18px);min-height:23px;margin-left:18px;border:0;border-radius:3px;background:transparent;color:var(--workbench-text);display:grid;grid-template-columns:32px minmax(0,1fr);gap:7px;align-items:center;padding:3px 6px;text-align:left;cursor:pointer}.workbench-search-hit:hover{background:#2a2d2e}.workbench-search-hit span:last-child{overflow:hidden;text-overflow:ellipsis;white-space:nowrap}.workbench-search-line{color:var(--workbench-muted);font-variant-numeric:tabular-nums;text-align:right}.workbench-search-more{margin:2px 0 0 56px;color:var(--workbench-muted);font-size:11px}.workbench-breadcrumb{height:24px;border-bottom:1px solid var(--workbench-border);background:#1e1e1e;color:var(--workbench-muted);display:flex;align-items:center;gap:6px;padding:0 12px;min-width:0;overflow:hidden;font-size:12px}.workbench-breadcrumb-part{display:inline-flex;align-items:center;gap:6px;min-width:0;max-width:220px;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}.workbench-breadcrumb-sep{color:#666}.workbench-editor-area{grid-column:3 / 4;grid-row:1 / 2;min-width:0;min-height:0;display:grid;grid-template-rows:35px 1fr;background:var(--workbench-bg)}.workbench-command-center{height:35px;border:0;border-bottom:1px solid var(--workbench-border);display:grid;place-items:center;color:var(--workbench-muted);font-size:12px;background:#1b1b1c;cursor:pointer}.workbench-command-center:hover{color:#d4d4d4;background:#202022}.workbench-editor-stack{min-height:0;display:grid;grid-template-rows:35px 24px minmax(0,1fr)}.workbench-workbench.has-bottom-panel .workbench-editor-stack{grid-template-rows:35px 24px minmax(0,1fr) 172px}.workbench-editor-stack:has(.workbench-ai-dirty-banner){grid-template-rows:35px 24px auto minmax(0,1fr)}.workbench-workbench.has-bottom-panel .workbench-editor-stack:has(.workbench-ai-dirty-banner){grid-template-rows:35px 24px auto minmax(0,1fr) 172px}.workbench-tabs{height:35px;border-bottom:1px solid var(--workbench-border);display:flex;align-items:stretch;min-width:0;overflow:hidden;background:#252526}.workbench-tab{min-width:120px;max-width:240px;border:0;border-right:1px solid var(--workbench-border);border-top:1px solid transparent;background:#2d2d2d;color:var(--workbench-text);display:flex;align-items:center;gap:7px;padding:0 9px;cursor:pointer}.workbench-tab.active{background:var(--workbench-bg);border-top-color:var(--workbench-blue);color:#fff}.workbench-tab-name{overflow:hidden;text-overflow:ellipsis;white-space:nowrap}.workbench-tab-close{margin-left:auto;border:0;color:inherit;background:transparent;width:18px;height:18px;border-radius:3px;display:grid;place-items:center;cursor:pointer}.workbench-tab-close:hover{background:#444}.workbench-dirty-dot{color:#fff;margin-right:6px}.workbench-tab-save{margin-left:auto;height:18px;padding:0 7px;border-radius:5px;border:1px solid var(--border-subtle, #3a3a40);background:#7c4dff29;color:#d9ccff;display:none;align-items:center;font-size:11px}.workbench-tab.active .workbench-tab-save,.workbench-tab:hover .workbench-tab-save{display:inline-flex}.workbench-tab:has(.workbench-tab-save) .workbench-tab-close{margin-left:2px}.workbench-editor-content{min-height:0;position:relative}.workbench-no-editor{height:100%;display:grid;place-content:center;gap:12px;justify-items:center;color:var(--workbench-muted);background:var(--workbench-bg)}.workbench-no-editor p{margin:0}.workbench-welcome{position:relative;height:100%;overflow:auto;padding:clamp(28px,7vh,92px) clamp(24px,6vw,88px);box-sizing:border-box;background:var(--workbench-bg);color:var(--workbench-text)}.workbench-welcome-grid{width:min(100%,980px);margin:0 auto;display:grid;grid-template-columns:repeat(auto-fit,minmax(260px,1fr));gap:clamp(24px,5vw,80px);align-items:start}.workbench-welcome h1{margin:0;font-weight:400;font-size:34px;color:#d4d4d4}.workbench-welcome h2{margin:28px 0 12px;font-size:20px;font-weight:600;color:#d4d4d4}.workbench-welcome-subtitle{color:#a7a7a7;font-size:16px;margin-top:2px}.workbench-start-link{display:flex;align-items:center;gap:8px;border:0;background:transparent;color:var(--workbench-blue-2);padding:5px 0;cursor:pointer;text-align:left}.workbench-start-link:hover{text-decoration:underline}.workbench-walkthrough-card{min-height:54px;border:1px solid transparent;border-radius:4px;background:#2d2d2d;color:var(--workbench-text);margin:8px 0;padding:10px 14px;box-sizing:border-box}.workbench-walkthrough-card strong{display:block;color:#d4d4d4;margin-bottom:4px}.workbench-welcome-checkbox{position:absolute;left:50%;bottom:34px;transform:translate(-50%);display:flex;align-items:center;gap:8px;color:#bdbdbd;white-space:nowrap}.workbench-secondary{grid-column:4 / 5;grid-row:1 / 2;min-width:0;min-height:0;background:var(--workbench-side);border-left:1px solid var(--workbench-border);display:grid;grid-template-rows:35px minmax(0,1fr);position:relative}.workbench-secondary-resizer{position:absolute;left:-6px;top:0;bottom:0;width:12px;cursor:col-resize;z-index:4;touch-action:none}.workbench-secondary-resizer:after{content:\"\";position:absolute;top:0;bottom:0;left:50%;width:2px;transform:translate(-50%);background:transparent;transition:background .12s ease}.workbench-secondary-resizer:hover:after,.workbench-secondary-resizer:active:after{background:var(--workbench-blue)}.workbench-secondary-tabs{display:flex;align-items:end;justify-content:space-between;border-bottom:1px solid var(--workbench-border);padding-left:12px;padding-right:8px;min-width:0;overflow:visible;position:relative;z-index:5}.workbench-secondary-tab-group{display:flex;align-items:end}.workbench-secondary-tab{height:34px;border:0;border-bottom:2px solid transparent;background:transparent;color:var(--workbench-muted);font-size:11px;font-weight:700;letter-spacing:.04em;cursor:pointer;padding:0 10px}.workbench-secondary-tab.active{color:#fff;border-bottom-color:var(--workbench-blue)}.workbench-secondary-actions{display:flex;align-items:center;gap:2px;height:34px}.workbench-secondary-actions button{width:26px;height:26px;border:0;border-radius:3px;color:var(--workbench-muted);background:transparent;display:grid;place-items:center;cursor:pointer}.workbench-secondary-actions button:hover{background:#2d2d2d;color:#fff}.workbench-secondary-actions button.is-active{background:var(--tytus-hover);border-color:var(--border-focus, rgba(124, 77, 255, .55));color:#fff}.workbench-history-portal{position:fixed;z-index:2147483600;width:440px;max-width:calc(100vw - 16px);max-height:min(540px,calc(100vh - 80px));background:var(--bg-elevated, #1f1f22);border:1px solid var(--border-subtle, #34343a);border-radius:10px;box-shadow:0 18px 44px #0000008c;display:flex;flex-direction:column;overflow:hidden;color:var(--text-primary, #e6e6ea);font-family:inherit}.workbench-history-portal-search{display:flex;align-items:center;gap:8px;padding:10px 12px;border-bottom:1px solid var(--border-subtle, #34343a);color:var(--workbench-muted, #8a8a93)}.workbench-history-portal-search input{flex:1;min-width:0;background:transparent;border:0;outline:0;color:var(--text-primary, #e6e6ea);font-size:13px;font-family:inherit;padding:2px 0}.workbench-history-portal-search input::placeholder{color:var(--workbench-muted, #8a8a93)}.workbench-history-portal-list{flex:1;min-height:0;overflow-y:auto;padding:6px;display:flex;flex-direction:column;gap:1px}.workbench-history-portal-group{padding:8px 10px 4px;font-size:10px;font-weight:700;text-transform:uppercase;letter-spacing:.08em;color:var(--workbench-muted, #8a8a93)}.workbench-history-portal-item{position:relative;width:100%;min-height:44px;border-radius:7px;font-size:13px;cursor:pointer;display:grid;grid-template-columns:minmax(0,1fr) auto auto;grid-template-rows:auto;column-gap:10px;align-items:center;padding:8px 12px;box-sizing:border-box;user-select:none}.workbench-history-portal-item:hover{background:#7c4dff24}.workbench-history-portal-item.active{background:#7c4dff38}.workbench-history-portal-title{grid-column:1 / 2;min-width:0;overflow:hidden;text-overflow:ellipsis;white-space:nowrap;font-weight:500;color:var(--text-primary, #e6e6ea);display:flex;align-items:center;gap:6px}.workbench-history-portal-badge{flex:0 0 auto;display:inline-grid;place-items:center;height:18px;padding:0 6px;border-radius:10px;background:#7c4dff38;color:#d9c8ff;font-size:9px;font-weight:700;letter-spacing:.06em;text-transform:uppercase}.workbench-history-portal-item small{grid-column:2 / 3;color:var(--workbench-muted, #8a8a93);font-size:11px;white-space:nowrap}.workbench-history-portal-delete{grid-column:3 / 4;width:26px;height:26px;display:inline-grid;place-items:center;border:0;border-radius:6px;background:transparent;color:var(--workbench-muted, #8a8a93);cursor:pointer;opacity:0;transition:opacity .12s ease,background .12s ease,color .12s ease}.workbench-history-portal-item:hover .workbench-history-portal-delete{opacity:1}.workbench-history-portal-delete:hover{background:#f443362e;color:#f44336}.workbench-history-portal-empty{padding:18px 12px;text-align:center;color:var(--workbench-muted, #8a8a93);font-size:12px}.workbench-history-portal-footer{display:flex;gap:14px;padding:8px 12px;border-top:1px solid var(--border-subtle, #34343a);font-size:11px;color:var(--workbench-muted, #8a8a93)}.workbench-history-portal-footer kbd{display:inline-grid;place-items:center;min-width:18px;height:18px;padding:0 4px;border:1px solid var(--border-subtle, #34343a);border-radius:4px;background:#ffffff0a;font-family:ui-monospace,SFMono-Regular,Menlo,Monaco,Consolas,monospace;font-size:10px;color:var(--text-primary, #e6e6ea);margin-right:4px}.workbench-chat-empty{height:100%;display:grid;place-items:center;text-align:center;padding:24px;box-sizing:border-box;color:#bdbdbd}.workbench-chat-empty svg{color:#cfcfcf;margin-bottom:16px}.workbench-chat-empty h3{margin:0 0 8px;font-size:15px;color:var(--workbench-text)}.workbench-chat-empty p{margin:4px auto;max-width:260px;line-height:1.45}.workbench-chat-empty-link{color:var(--workbench-blue-2)}.workbench-chat-wrap{min-width:0;min-height:0;overflow:hidden;display:grid;grid-template-rows:auto minmax(0,1fr) auto}.workbench-chat-threadbar{display:grid;grid-template-columns:minmax(0,1fr) minmax(0,auto);align-items:center;gap:6px;padding:10px 12px 0;background:var(--workbench-side);min-width:0}.workbench-chat-thread-title{min-width:0;overflow:hidden;text-overflow:ellipsis;white-space:nowrap;color:var(--workbench-text);font-size:12px;font-weight:600}.workbench-chat-thread-actions{display:inline-flex;align-items:center;gap:4px}.workbench-chat-iconbtn{width:26px;height:26px;display:inline-grid;place-items:center;border:1px solid transparent;border-radius:6px;background:transparent;color:var(--workbench-muted, #9a9aa3);cursor:pointer;list-style:none;transition:background .12s ease,color .12s ease,border-color .12s ease}.workbench-chat-iconbtn::-webkit-details-marker{display:none}.workbench-chat-iconbtn:hover:not(:disabled){background:#7c4dff24;color:#fff;border-color:#7c4dff66}.workbench-chat-iconbtn:disabled{opacity:.45;cursor:not-allowed}.workbench-chat-iconmenu{position:relative}.workbench-chat-iconmenu[open]>summary{background:#7c4dff2e;color:#fff;border-color:#7c4dff8c}.workbench-chat-history-pop{position:absolute;z-index:40;top:calc(100% + 6px);right:0;min-width:280px;max-width:360px;max-height:400px;overflow-y:auto;padding:6px;background:var(--bg-elevated, #1f1f22);border:1px solid var(--border-subtle, #34343a);border-radius:8px;box-shadow:0 12px 28px #00000073;display:flex;flex-direction:column;gap:2px}.workbench-chat-history-header{padding:6px 8px 4px;font-size:10px;font-weight:700;text-transform:uppercase;letter-spacing:.08em;color:var(--workbench-muted, #8a8a93)}.workbench-chat-history-pop button{height:auto;min-height:34px;border:0;background:transparent;color:var(--text-primary, #e6e6ea);text-align:left;padding:6px 10px;border-radius:6px;font-size:12px;cursor:pointer;display:flex;flex-direction:column;align-items:flex-start;gap:2px}.workbench-chat-history-pop button:hover{background:#7c4dff29}.workbench-chat-history-pop button.active{background:#7c4dff3d;color:#fff}.workbench-chat-history-title{width:100%;overflow:hidden;text-overflow:ellipsis;white-space:nowrap;font-weight:500}.workbench-chat-history-pop button small{color:var(--workbench-muted, #8a8a93);font-size:10px}.workbench-chat-transcript{overflow-y:auto;overflow-x:hidden;padding:12px;min-width:0;min-height:0;user-select:text;-webkit-user-select:text}.workbench-chat-message{border:1px solid var(--workbench-border-strong);background:#252526;border-radius:4px;padding:12px;margin-bottom:10px;line-height:1.45;user-select:text;-webkit-user-select:text;min-width:0;overflow-wrap:anywhere}.workbench-chat-message.user{border-color:#315f8a}.workbench-chat-message-actions{display:flex;flex-wrap:wrap;justify-content:flex-end;gap:5px;margin-top:12px;padding-top:10px;border-top:1px solid rgba(255,255,255,.06)}.workbench-chat-message-actions button{height:26px;border:1px solid rgba(255,255,255,.08);border-radius:999px;background:#ffffff09;color:var(--text-secondary, #b9b9c1);cursor:pointer;display:inline-flex;align-items:center;gap:5px;padding:0 9px;font-size:11px;font-weight:600;line-height:1}.workbench-chat-message-actions button:hover:not(:disabled){background:#ffffff12;border-color:var(--border-focus, rgba(124, 77, 255, .55));color:#fff}.workbench-chat-message-actions button:disabled{cursor:not-allowed;opacity:.42}.workbench-chat-message-actions button.regen{color:#f2d2ec;border-color:#d85cae4d;background:#d85cae14}.workbench-chat-message-actions button.regen:hover:not(:disabled){border-color:#eb72bda6;background:#d85cae29}.workbench-rich-body{margin-top:6px;color:var(--workbench-text);user-select:text;-webkit-user-select:text;cursor:text}.workbench-rich-markdown{user-select:text;-webkit-user-select:text}.workbench-rich-markdown>:first-child{margin-top:0}.workbench-rich-markdown>:last-child{margin-bottom:0}.workbench-rich-markdown p{margin:0 0 9px}.workbench-rich-markdown ul,.workbench-rich-markdown ol{margin:7px 0 10px 18px;padding:0}.workbench-rich-markdown li{margin:3px 0}.workbench-rich-markdown h1,.workbench-rich-markdown h2,.workbench-rich-markdown h3{margin:12px 0 7px;color:#f2f2f4;line-height:1.2}.workbench-rich-markdown h1{font-size:18px}.workbench-rich-markdown h2{font-size:16px}.workbench-rich-markdown h3{font-size:14px}.workbench-rich-markdown a{color:var(--workbench-blue-2)}.workbench-rich-markdown code{border:1px solid rgba(255,255,255,.08);border-radius:4px;background:#00000040;color:#e7d8ff;padding:1px 4px;font-family:ui-monospace,SFMono-Regular,Menlo,Monaco,Consolas,Liberation Mono,monospace;font-size:.94em;user-select:text;-webkit-user-select:text}.workbench-rich-code{overflow:hidden;margin:10px 0 12px;border:1px solid rgba(255,255,255,.09);border-radius:8px;background:#171719;user-select:text;-webkit-user-select:text}.workbench-rich-code-head{min-height:30px;display:flex;align-items:center;justify-content:space-between;gap:8px;padding:0 8px 0 10px;border-bottom:1px solid rgba(255,255,255,.07);background:#232326}.workbench-rich-code-head span{color:var(--workbench-muted);font-size:11px;font-weight:700;letter-spacing:.04em;text-transform:uppercase;user-select:text;-webkit-user-select:text}.workbench-rich-code-head button{height:22px;display:inline-flex;align-items:center;gap:4px;border:1px solid rgba(255,255,255,.08);border-radius:999px;background:#ffffff0a;color:#d6d6dc;cursor:pointer;font-size:11px;user-select:none;-webkit-user-select:none}.workbench-rich-code-head button:hover{border-color:#7c4dff8c;background:#7c4dff29;color:#fff}.workbench-rich-code pre{margin:0;padding:12px;overflow:auto;color:#e6e6ea;background:#171719;font-size:12px;line-height:1.5;user-select:text;-webkit-user-select:text;cursor:text}.workbench-rich-code code{padding:0;border:0;background:transparent;color:inherit;font-family:ui-monospace,SFMono-Regular,Menlo,Monaco,Consolas,Liberation Mono,monospace;white-space:pre;user-select:text;-webkit-user-select:text;cursor:text}.workbench-chat-composer{padding:0 12px 12px;border-top:1px solid var(--workbench-border);background:var(--workbench-side);min-width:0}.workbench-chat-tip{margin:10px 0 6px;padding:7px 9px;border:1px solid var(--workbench-border-strong);border-radius:3px 3px 0 0;color:var(--workbench-muted);font-size:12px;background:#252526;display:grid;grid-template-columns:auto minmax(0,1fr) minmax(0,auto);align-items:center;gap:8px;min-width:0}.workbench-chat-tip span{color:var(--workbench-blue-2);font-weight:700}.workbench-chat-tip em{min-width:0;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}.workbench-chat-tip strong{min-width:0;overflow:hidden;text-overflow:ellipsis;white-space:nowrap;color:var(--workbench-text);font-weight:600}.workbench-chat-tip em{color:var(--workbench-muted);font-style:normal}.workbench-chat-box{border:1px solid #3c3c3c;border-radius:2px;background:#1e1e1e;box-shadow:0 12px 28px #0003}.workbench-chat-attachments{min-height:28px;display:flex;align-items:center;flex-wrap:wrap;row-gap:6px;gap:6px;padding:5px 8px 0;min-width:0}.workbench-chat-attachments button,.workbench-chat-send,.workbench-chat-mode{border:0;border-radius:3px;background:#2d2d2d;color:var(--workbench-text);cursor:pointer}.workbench-chat-attachments button{width:24px;height:24px;display:grid;place-items:center}.workbench-chat-chip{max-width:270px;display:inline-flex;align-items:center;gap:5px;padding:3px 7px;border-radius:3px;background:#2a2d2e;color:#cfcfcf;font-size:12px;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}.workbench-chat-chip-button{height:23px;border:1px solid var(--border-subtle, #34343a)!important;border-radius:999px!important;background:#24272a!important;color:#d7d7d7!important;padding:0 9px;width:auto!important;font-size:11px}.workbench-chat-chip-button:hover:not(:disabled){border-color:var(--border-focus, rgba(124, 77, 255, .55))!important;background:#2e3338!important}.workbench-chat-chip-button:disabled{cursor:not-allowed;opacity:.45}.workbench-chat-textarea{width:100%;min-height:92px;max-height:220px;resize:vertical;border:0;outline:none;background:transparent;color:var(--workbench-text);padding:7px 10px;box-sizing:border-box;font:inherit;line-height:1.45}.workbench-chat-toolbar{height:32px;display:grid;grid-template-columns:minmax(0,auto) auto 1fr auto;align-items:center;gap:6px;padding:0 8px 8px;min-width:0}.workbench-chat-mode{height:24px;padding:0 8px;display:inline-flex;align-items:center;gap:4px;color:#cfcfcf}.workbench-chat-send{width:28px;height:26px;display:grid;place-items:center;background:#2d2d2d;color:#fff}.workbench-chat-send:disabled{cursor:not-allowed;opacity:.5}.workbench-chat-send.stop{background:#b64a4a}.workbench-settings-tab{height:100%;min-height:0;overflow:auto;background:var(--workbench-bg);color:var(--workbench-text)}.workbench-settings-page{width:min(100%,980px);min-height:100%;margin:0 auto;display:flex;flex-direction:column;overflow:hidden;border-left:1px solid var(--workbench-border);border-right:1px solid var(--workbench-border);background:var(--workbench-panel)}.workbench-settings-header{height:48px;min-height:48px;display:flex;align-items:center;gap:9px;padding:0 16px;color:var(--workbench-text);border-bottom:1px solid var(--workbench-border);background:var(--workbench-side)}.workbench-settings-header svg{color:var(--workbench-blue-2)}.workbench-settings-header button{margin-left:auto;width:28px;height:28px;border:0;border-radius:4px;background:transparent;color:var(--workbench-muted);cursor:pointer;display:grid;place-items:center}.workbench-settings-header button:hover{background:#2d2d2d;color:#fff}.workbench-settings-body{overflow:visible;padding:clamp(14px,2.4vw,24px);display:grid;grid-template-columns:repeat(auto-fit,minmax(280px,1fr));gap:14px}.workbench-settings-section{border:1px solid var(--workbench-border);border-radius:8px;background:#202020;padding:14px}.workbench-settings-section h3{margin:0 0 6px;font-size:13px;color:var(--workbench-text)}.workbench-settings-section p{margin:0 0 14px;color:var(--workbench-muted);font-size:12px;line-height:1.45}.workbench-settings-label{display:grid;gap:6px;margin-top:12px;font-size:11px;font-weight:700;color:#bbb;text-transform:uppercase;letter-spacing:.04em}.workbench-settings-label select,.workbench-settings-label input{height:32px;border:1px solid var(--workbench-border-strong);border-radius:5px;background:#1b1b1b;color:var(--workbench-text);padding:0 10px;outline:none;font:inherit;text-transform:none;letter-spacing:normal}.workbench-settings-label input{font-family:ui-monospace,SFMono-Regular,Menlo,Monaco,Consolas,monospace}.workbench-settings-label select:focus,.workbench-settings-label input:focus{border-color:var(--workbench-blue)}.workbench-settings-note{margin-top:12px;padding:9px 10px;border:1px solid var(--workbench-border);border-radius:6px;background:#252526;color:var(--workbench-muted);font-size:12px}.workbench-settings-footer{min-height:48px;display:flex;align-items:center;justify-content:flex-end;gap:8px;padding:0 16px;border-top:1px solid var(--workbench-border);background:var(--workbench-side)}.workbench-settings-footer button{height:30px;border:1px solid var(--workbench-border-strong);border-radius:5px;background:#2d2d2d;color:var(--workbench-text);cursor:pointer;padding:0 12px}.workbench-settings-footer button:hover{background:#383838;color:#fff}.workbench-panel-list{overflow:auto;padding:12px}.workbench-panel-list.compact{padding:0}.workbench-output-card{border:1px solid var(--workbench-border-strong);background:#252526;border-radius:4px;padding:12px;margin-bottom:10px;user-select:text;-webkit-user-select:text}.workbench-output-head{display:flex;gap:8px;align-items:center;flex-wrap:wrap;margin-bottom:8px}.workbench-output-head strong{flex:1 1 160px;min-width:0;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}.workbench-output-head span{color:var(--workbench-muted);font-size:11px}.workbench-output-head button{height:22px;border:1px solid var(--border-subtle, #34343a);border-radius:6px;background:transparent;color:var(--workbench-muted);cursor:pointer;font-size:11px}.workbench-output-head button:hover{color:var(--workbench-text);background:var(--tytus-hover, #313131)}.workbench-output-head .workbench-output-edit-cta{border-color:#7c4dffb3;color:#fff;background:#7c4dff2e}.workbench-bottom-panel{min-height:0;border-top:1px solid var(--workbench-border);background:var(--workbench-bg);display:grid;grid-template-rows:32px minmax(0,1fr)}.workbench-bottom-tabs{height:32px;display:grid;grid-template-columns:repeat(3,auto) 1fr auto;align-items:end;gap:14px;padding:0 10px;border-bottom:1px solid var(--workbench-border);background:var(--bg-titlebar, #202022)}.workbench-bottom-tabs button{height:31px;border:0;border-bottom:2px solid transparent;background:transparent;color:var(--workbench-muted);font-size:11px;font-weight:700;letter-spacing:.04em;cursor:pointer}.workbench-bottom-tabs button.active{color:#fff;border-bottom-color:var(--workbench-blue)}.workbench-bottom-tabs button:last-child{width:26px;border-radius:4px;display:grid;place-items:center}.workbench-bottom-tabs button:last-child:hover{background:var(--tytus-hover, #313131);color:#fff}.workbench-bottom-body{min-height:0;overflow:auto;padding:12px}.workbench-terminal-placeholder{margin:10px 0 0;padding:10px 12px;border:1px solid var(--workbench-border-strong);border-radius:7px;background:#101010;color:#d7d7d7}.workbench-command-overlay{position:absolute;inset:0;z-index:30;background:#0000002e;display:flex;align-items:flex-start;justify-content:center;padding-top:18px}.workbench-command-palette{width:min(640px,calc(100% - 80px));border:1px solid #454545;border-radius:8px;background:#252526;box-shadow:0 18px 44px #00000073;overflow:hidden}.workbench-command-input{width:100%;height:38px;border:0;border-bottom:1px solid var(--workbench-border);outline:none;background:#1f1f1f;color:var(--workbench-text);padding:0 12px;box-sizing:border-box;font:inherit}.workbench-command-list{max-height:360px;overflow:auto;padding:6px}.workbench-command-item{width:100%;min-height:42px;border:0;border-radius:4px;background:transparent;color:var(--workbench-text);display:flex;flex-direction:column;align-items:flex-start;gap:2px;padding:7px 10px;text-align:left;cursor:pointer}.workbench-command-item:hover{background:#094771}.workbench-command-item:disabled{cursor:not-allowed;opacity:.45}.workbench-command-item small{color:var(--workbench-muted);font-size:11px}.workbench-edit-review-overlay{position:absolute;inset:0;z-index:40;background:#0000007a;display:grid;place-items:center;padding:24px}.workbench-edit-review{width:min(1180px,calc(100vw - 80px));height:min(760px,calc(100vh - 100px));border:1px solid var(--workbench-border-strong);border-radius:10px;background:#1e1e1e;box-shadow:0 24px 64px #0000008c;display:grid;grid-template-rows:auto auto minmax(0,1fr) auto;overflow:hidden}.workbench-edit-review.workspace{width:min(980px,calc(100vw - 80px))}.workbench-edit-review-head{height:48px;display:flex;align-items:center;justify-content:space-between;gap:12px;padding:0 14px;border-bottom:1px solid var(--workbench-border);background:var(--bg-titlebar, #202022)}.workbench-edit-review-head div{min-width:0;display:flex;flex-direction:column;gap:2px}.workbench-edit-review-head strong{color:var(--workbench-text)}.workbench-edit-review-head span{max-width:720px;overflow:hidden;text-overflow:ellipsis;white-space:nowrap;color:var(--workbench-muted);font-size:12px}.workbench-edit-review-head button{width:30px;height:30px;border:0;border-radius:6px;background:transparent;color:var(--workbench-muted);display:grid;place-items:center;cursor:pointer}.workbench-edit-review-head button:hover{background:var(--tytus-hover, #313131);color:#fff}.workbench-edit-review-meta{min-height:34px;display:flex;flex-wrap:wrap;align-items:center;gap:8px;padding:7px 14px;border-bottom:1px solid var(--workbench-border);color:var(--workbench-muted);font-size:12px}.workbench-edit-review-meta span{border:1px solid var(--border-subtle, #34343a);border-radius:999px;padding:3px 8px;background:#242424}.workbench-edit-review-grid{min-height:0;display:grid;grid-template-columns:minmax(0,1fr) minmax(0,1fr);gap:1px;background:var(--workbench-border)}.workbench-edit-review-pane{min-width:0;min-height:0;display:grid;grid-template-rows:32px minmax(0,1fr);background:#1e1e1e}.workbench-edit-review-pane h4{margin:0;padding:8px 12px;border-bottom:1px solid var(--workbench-border);color:var(--workbench-muted);font-size:11px;letter-spacing:.05em;text-transform:uppercase}.workbench-edit-review-pane.proposed h4{color:var(--workbench-blue-2)}.workbench-edit-review-pane pre{margin:0;padding:12px;overflow:auto;white-space:pre;color:#d7d7d7;font-family:var(--mono, \"SFMono-Regular\", Consolas, \"Liberation Mono\", monospace);font-size:12px;line-height:1.45}.workbench-workspace-patch-list{min-height:0;overflow:auto;padding:12px}.workbench-workspace-patch-card{border:1px solid var(--workbench-border-strong);border-radius:8px;background:#252526;margin-bottom:10px;overflow:hidden}.workbench-workspace-patch-card.skipped{border-color:#ffbe5c59}.workbench-workspace-patch-card header{min-height:34px;display:grid;grid-template-columns:minmax(0,1fr) auto;align-items:center;gap:10px;padding:8px 10px;border-bottom:1px solid var(--workbench-border)}.workbench-workspace-patch-card header strong{min-width:0;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}.workbench-workspace-patch-card header span{color:var(--workbench-muted);font-size:11px}.workbench-workspace-patch-card pre{max-height:260px;margin:0;padding:10px;overflow:auto;color:#d7d7d7;white-space:pre;font-family:var(--mono, \"SFMono-Regular\", Consolas, \"Liberation Mono\", monospace);font-size:11px;line-height:1.45}.workbench-edit-review-actions{height:52px;display:flex;justify-content:flex-end;align-items:center;gap:8px;padding:0 14px;border-top:1px solid var(--workbench-border);background:var(--bg-titlebar, #202022)}.workbench-button-primary{height:30px;border:0;border-radius:7px;background:var(--workbench-blue);color:#fff;padding:0 12px;cursor:pointer}.workbench-button-primary:hover{filter:brightness(1.08)}.workbench-ai-dirty-banner{display:flex;align-items:center;gap:8px;min-height:32px;padding:0 10px;border-bottom:1px solid rgba(255,184,77,.35);background:#ffb84d1f;color:#ffd28a;font-size:12px}.workbench-ai-dirty-banner span{min-width:0;flex:1;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}.workbench-ai-dirty-banner button{height:22px;border:1px solid rgba(255,210,138,.55);border-radius:5px;background:#ffffff0d;color:#ffd28a;cursor:pointer}.workbench-statusbar{grid-column:3 / 5;grid-row:2 / 3;height:22px;background:var(--workbench-blue);color:#fff;display:flex;align-items:center;gap:12px;padding:0 10px;font-size:12px;min-width:0}.workbench-status-spacer{flex:1}.workbench-problems-panel{height:152px;border-top:1px solid var(--workbench-border);background:var(--workbench-bg);display:grid;grid-template-rows:30px 1fr}.workbench-panel-tabs{display:flex;gap:18px;align-items:center;padding:0 12px;color:var(--workbench-muted);font-size:11px;font-weight:700}.workbench-panel-tabs span:first-child{color:#fff;border-bottom:1px solid var(--workbench-blue);height:29px;display:flex;align-items:center}.workbench-panel-body{padding:12px;color:var(--workbench-muted)}.workbench-empty-pane{padding:14px;color:var(--workbench-muted)}.workbench-extension-card{border:1px solid var(--workbench-border-strong);background:#252526;padding:12px;margin-bottom:8px}.workbench-extension-card strong{display:block;margin-bottom:4px}.workbench-inline-error{border:1px solid rgba(244,115,115,.35);background:#f473731f;color:#ffb4b4;padding:9px 10px;margin:8px 0;font-size:12px}.workbench-computer-hero{display:flex;gap:10px;border:1px solid var(--workbench-border-strong);background:#252526;padding:12px;margin-bottom:8px}.workbench-computer-hero strong{display:block;margin-bottom:4px}.workbench-team-switcher{display:grid;grid-template-columns:repeat(3,1fr);gap:4px;padding:4px;margin:8px 0 12px;border:1px solid rgba(255,255,255,.08);border-radius:10px;background:#0000002e}.workbench-team-switcher button{border:0;border-radius:7px;min-height:28px;color:var(--workbench-muted);background:transparent;font-size:11px;font-weight:700;cursor:pointer}.workbench-team-switcher button.active{color:#f3f3f4;background:linear-gradient(135deg,#7c4dff6b,#569cd638)}.workbench-computer-refresh{width:100%;display:inline-flex;align-items:center;justify-content:center;gap:8px;margin-bottom:12px}.workbench-computer-list{display:grid;gap:8px;margin:8px 0 14px}.workbench-computer-explainer{border:1px solid rgba(86,156,214,.24);background:#569cd612;color:var(--workbench-muted);display:grid;gap:6px;padding:9px 10px;margin:10px 0 12px;font-size:11px;line-height:1.4}.workbench-computer-explainer strong{color:var(--workbench-text);font-size:12px}.workbench-computer-explainer b{color:var(--workbench-text);font-weight:700}.workbench-computer-explainer code{color:#d7ba7d;background:#0003;border:1px solid rgba(255,255,255,.08);padding:0 4px}.workbench-computer-list.compact{gap:5px}.workbench-computer-context-card{border:1px solid rgba(124,77,255,.28);background:#7c4dff14;border-radius:var(--tytus-radius-sm, 8px);padding:9px 10px;display:grid;gap:4px;font-size:12px;min-width:0}.workbench-computer-context-card.mission{border-color:#569cd65c;background:linear-gradient(135deg,#569cd61f,#7c4dff14)}.workbench-computer-context-card strong,.workbench-computer-context-card span{overflow:hidden;text-overflow:ellipsis}.workbench-computer-context-card span{color:var(--workbench-muted);font-size:11px}.workbench-team-assignment-list{display:grid;gap:6px;margin:8px 0 12px}.workbench-team-assignment-summary,.workbench-team-assignment-row{border:1px solid rgba(255,255,255,.08);background:#ffffff09;border-radius:8px;padding:8px;min-width:0}.workbench-team-assignment-summary{border-color:#7c4dff52;background:#7c4dff1a}.workbench-team-assignment-summary strong,.workbench-team-assignment-summary span,.workbench-team-assignment-row strong,.workbench-team-assignment-row span{display:block;min-width:0;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}.workbench-team-assignment-summary span,.workbench-team-assignment-row span{color:var(--workbench-muted);font-size:10px;margin-top:2px}.workbench-team-assignment-row{display:grid;grid-template-columns:minmax(0,1fr) auto;gap:8px;align-items:center}.workbench-team-assignment-row em{color:#9af0b4;font-size:10px;font-style:normal;font-weight:800;text-transform:uppercase}.workbench-computer-presets{display:grid;grid-template-columns:1fr;gap:6px;margin-bottom:8px}.workbench-computer-presets .workbench-button-subtle{justify-content:flex-start;text-align:left}.workbench-computer-actions{display:grid;grid-template-columns:1fr;gap:6px}.workbench-computer-actions .workbench-button-subtle{justify-content:center}.workbench-computer-job-prompt{width:100%;min-height:92px;box-sizing:border-box;resize:vertical;background:var(--bg-input, #242426);color:var(--workbench-text);border:1px solid var(--border-subtle, #35353b);padding:8px;font:inherit;font-size:12px}.workbench-computer-job-log{max-height:180px;overflow:auto;white-space:pre-wrap;word-break:break-word;background:#111;border:1px solid var(--workbench-border);color:var(--workbench-muted);padding:8px;font-size:11px}.workbench-computer-card{border:1px solid var(--border-subtle, #34343a);background:var(--bg-elevated, #2b2b2d);color:var(--workbench-text);padding:10px;display:grid;gap:8px}.workbench-computer-card-head{display:flex;align-items:flex-start;justify-content:space-between;gap:10px}.workbench-computer-card-head strong,.workbench-computer-card-head span{display:block;min-width:0;overflow:hidden;text-overflow:ellipsis}.workbench-computer-card-head span{color:var(--workbench-muted);font-size:11px;margin-top:2px}.workbench-computer-pill{flex:0 0 auto;border:1px solid var(--workbench-border-strong);color:var(--workbench-muted);border-radius:999px;padding:2px 7px;font-size:10px;text-transform:uppercase;letter-spacing:.04em}.workbench-computer-pill.available,.workbench-computer-pill.ready{color:#9af0b4;border-color:#9af0b459;background:#9af0b414}.workbench-computer-pill.degraded,.workbench-computer-pill.needs-setup,.workbench-computer-pill.needs_setup{color:#ffd48a;border-color:#ffd48a59;background:#ffd48a14}.workbench-computer-pill.missing{color:#ff9e9e;border-color:#ff9e9e59;background:#ff9e9e14}.workbench-resource-warnings{display:grid;gap:4px;margin:6px 0 12px}.workbench-resource-warnings span{color:#ffd48a;font-size:11px;border:1px solid rgba(255,212,138,.22);background:#ffd48a12;padding:5px 7px}.workbench-resource-row{min-width:0;display:grid;grid-template-columns:minmax(0,1fr) auto;gap:8px;align-items:center;border:1px solid rgba(255,255,255,.07);background:#ffffff06;padding:7px 8px}.workbench-resource-row strong,.workbench-resource-row span{display:block;min-width:0;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}.workbench-resource-row div>span{color:var(--workbench-muted);font-size:10px;margin-top:2px}.workbench-resource-row-actions{display:flex;align-items:center;justify-content:flex-end;gap:4px;min-width:0}.workbench-resource-row-actions .workbench-button-subtle{min-height:24px;padding:2px 7px;font-size:10px}.workbench-mission-list,.workbench-task-graph{display:grid;gap:6px;margin-bottom:12px}.workbench-mission-row,.workbench-task-card{width:100%;min-width:0;border:1px solid rgba(255,255,255,.08);background:#ffffff09;color:var(--workbench-text);border-radius:8px;padding:8px;text-align:left;cursor:pointer}.workbench-mission-row:hover,.workbench-task-card:hover,.workbench-mission-row.active{border-color:#7c4dff73;background:#7c4dff1f}.workbench-mission-row strong,.workbench-mission-row span,.workbench-task-card strong,.workbench-task-card em,.workbench-task-card small{display:block;min-width:0;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}.workbench-mission-row span,.workbench-task-card em,.workbench-task-card small{color:var(--workbench-muted);font-size:10px;margin-top:2px;font-style:normal}.workbench-task-card{display:grid;grid-template-columns:22px minmax(0,1fr);column-gap:8px;align-items:center}.workbench-task-card>span{width:22px;height:22px;display:inline-flex;align-items:center;justify-content:center;border-radius:50%;background:#7c4dff2e;color:#d9ccff;font-size:11px;font-weight:800}.workbench-task-card em{grid-column:2}.workbench-task-card small{grid-column:2;color:#9fbfe8}.workbench-computer-triggers{display:flex;flex-wrap:wrap;gap:4px}.workbench-computer-triggers span{border:1px solid var(--workbench-border);color:var(--workbench-muted);padding:2px 6px;font-size:10px}.workbench-extension-row{width:100%;min-height:58px;border:1px solid var(--border-subtle, #34343a);border-radius:8px;background:var(--bg-elevated, #2b2b2d);color:var(--workbench-text);display:grid;grid-template-columns:22px minmax(0,1fr) auto;align-items:center;gap:10px;padding:9px 10px;margin-bottom:8px;box-sizing:border-box}.workbench-extension-row strong,.workbench-extension-row span{display:block;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}.workbench-extension-row span{color:var(--workbench-muted);font-size:12px;margin-top:2px}.workbench-extension-row em{color:var(--workbench-muted);font-size:11px;font-style:normal}.workbench-badge{display:inline-flex;align-items:center;border-radius:10px;background:#333;color:#aaa;padding:2px 8px;font-size:11px}@media(max-width:980px){.workbench-workbench{grid-template-columns:48px 260px minmax(0,1fr) 0}.workbench-secondary{display:none}.workbench-statusbar{grid-column:3 / 4}}.workbench-workbench{grid-template-columns:48px var(--workbench-primary-width, 300px) minmax(0,1fr) var(--workbench-secondary-width, 520px);--tytus-radius-sm: 7px;--tytus-radius-md: 10px;--tytus-surface: var(--bg-window, #1f1f1f);--tytus-surface-2: var(--bg-panel, #252526);--tytus-hover: var(--bg-hover, #313131);--tytus-accent: var(--accent-primary, #7c4dff);--tytus-accent-hover: var(--accent-primary-hover, #9068ff)}.workbench-workbench.no-primary{grid-template-columns:48px 0 minmax(0,1fr) var(--workbench-secondary-width, 460px)}.workbench-workbench.no-secondary{grid-template-columns:48px var(--workbench-primary-width, 300px) minmax(0,1fr)}.workbench-workbench.no-primary.no-secondary{grid-template-columns:48px 0 minmax(0,1fr)}.workbench-workbench.no-secondary .workbench-statusbar{grid-column:3 / 4}.workbench-button-blue,.workbench-button-subtle,.workbench-input,.workbench-file-row,.workbench-tree-row,.workbench-folder-row,.workbench-walkthrough-card,.workbench-output-card,.workbench-chat-message,.workbench-chat-tip,.workbench-chat-box,.workbench-extension-card,.workbench-extension-row{border-radius:var(--tytus-radius-sm)}.workbench-button-blue{background:linear-gradient(135deg,var(--tytus-accent),#d85cae);box-shadow:0 8px 22px #7c4dff2e;font-weight:650}.workbench-button-blue:hover{background:linear-gradient(135deg,var(--tytus-accent-hover),#eb72bd)}.workbench-button-subtle,.workbench-chat-attachments button,.workbench-chat-mode,.workbench-secondary-actions button,.workbench-tab-close{border:1px solid var(--border-subtle, #34343a);background:var(--bg-elevated, #2b2b2d)}.workbench-button-subtle:hover,.workbench-chat-attachments button:hover,.workbench-chat-mode:hover,.workbench-secondary-actions button:hover{background:var(--tytus-hover);border-color:var(--border-focus, rgba(124, 77, 255, .55))}.workbench-input,.workbench-chat-textarea{background:var(--bg-input, #242426);border-color:var(--border-subtle, #35353b)}.workbench-input:focus,.workbench-chat-box:focus-within{border-color:var(--tytus-accent);box-shadow:0 0 0 1px #7c4dff73,0 0 0 4px #7c4dff1a}.workbench-editor-action{width:34px;height:28px;align-self:center;margin-right:6px;border:1px solid transparent;border-radius:7px;background:transparent;color:var(--workbench-muted);display:grid;place-items:center;cursor:pointer}.workbench-editor-action:hover,.workbench-editor-action.active{color:#fff;background:var(--bg-hover, #313131);border-color:var(--border-subtle, #3a3a40)}.workbench-editor-single,.workbench-editor-split{width:100%;height:100%;min-height:0}.workbench-editor-single{display:grid;grid-template-columns:minmax(0,1fr)}.workbench-editor-split{display:grid;grid-template-columns:minmax(0,1fr) minmax(320px,42%)}.workbench-editor-pane{min-width:0;min-height:0;height:100%}.workbench-markdown-preview{min-width:0;min-height:0;border-left:1px solid var(--workbench-border);background:var(--bg-window, #1f1f1f);display:grid;grid-template-rows:30px minmax(0,1fr)}.workbench-preview-title{height:30px;display:flex;align-items:center;gap:6px;padding:0 10px;color:var(--text-secondary, #aaa);font-size:12px;border-bottom:1px solid var(--border-subtle, #333);background:var(--bg-titlebar, #242426)}.workbench-preview-body{overflow:auto;padding:20px 26px 32px;color:var(--text-primary, #ddd)}.workbench-preview-body h1,.workbench-preview-body h2,.workbench-preview-body h3{color:var(--text-primary, #eee)}.workbench-preview-body p,.workbench-preview-body li{line-height:1.65}.workbench-secondary-tabs{background:var(--bg-titlebar, #202022)}.workbench-primary-resizer:hover,.workbench-secondary-resizer:hover{background:var(--tytus-accent)}.workbench-chat-composer{padding:0 12px 12px}.workbench-chat-tip{border-radius:var(--tytus-radius-sm) var(--tytus-radius-sm) 0 0}.workbench-chat-box{border-radius:0 0 var(--tytus-radius-md) var(--tytus-radius-md)}.workbench-chat-send{width:32px;height:32px;display:inline-grid;place-items:center;border-radius:8px;background:var(--bg-elevated, #232326);color:var(--text-secondary, #8a8a93);border:1px solid var(--border-subtle, #34343a);box-shadow:none;transition:transform .12s ease,background .12s ease,border-color .12s ease,box-shadow .12s ease,color .12s ease}.workbench-chat-send svg{display:block;width:16px;height:16px;stroke-width:2.4}.workbench-chat-send:disabled{opacity:1;cursor:not-allowed;background:transparent;color:var(--text-secondary, #6a6a72);border-color:var(--border-subtle, #34343a)}.workbench-chat-send.ready{background:linear-gradient(135deg,#8b5cff,#d85cae);color:#fff;border-color:transparent;box-shadow:0 6px 14px #7c4dff4d,0 1px 2px #0003}.workbench-chat-send.ready:hover{transform:translateY(-1px);background:linear-gradient(135deg,#9a6fff,#e472be);box-shadow:0 8px 20px #7c4dff6b,0 2px 4px #0000003d}.workbench-chat-send.ready:active{transform:translateY(0);box-shadow:0 3px 8px #7c4dff4d,inset 0 1px 2px #0003}.workbench-chat-send.stop{background:linear-gradient(135deg,#b64a4a,#ef6f6f);color:#fff;border-color:transparent;box-shadow:0 6px 14px #b64a4a4d,0 1px 2px #0003}.workbench-chat-send.stop:hover{transform:translateY(-1px);background:linear-gradient(135deg,#c75555,#f48080);box-shadow:0 8px 20px #b64a4a6b,0 2px 4px #0000003d}@media(max-width:1200px){.workbench-editor-split{grid-template-columns:minmax(0,1fr)}.workbench-markdown-preview{display:none}}.workbench-chat-context-select{height:24px;border:1px solid var(--workbench-border-strong);border-radius:999px;background:#1f1f1f;color:var(--workbench-text);font-size:11px;padding:0 8px;outline:none}.workbench-chat-chip.muted{color:var(--workbench-muted)}.workbench-chat-chip.warn{color:#f6c177;border:1px solid rgba(246,193,119,.35)}.workbench-chat-chip small{color:#f6c177;font-size:10px;margin-left:4px}.workbench-chat-chip-open,.workbench-chat-chip-remove{width:auto!important;height:auto!important;border:0!important;background:transparent!important;color:inherit!important;padding:0!important;display:inline-flex!important;align-items:center;cursor:pointer}.workbench-chat-chip-remove{margin-left:4px;opacity:.75}.workbench-chat-chip-open:hover,.workbench-chat-chip-remove:hover{opacity:1;color:#fff!important}.workbench-chat-jump{position:sticky;bottom:8px;display:block;margin:8px auto 0;height:26px;border:1px solid var(--workbench-border-strong);border-radius:999px;background:#2d2d30;color:var(--workbench-text);font-size:11px;padding:0 12px;cursor:pointer;box-shadow:0 6px 16px #00000047}.workbench-chat-jump:hover{background:#3a3a3d;color:#fff}.workbench-chat-generate-patch{margin:6px 8px 0;min-height:28px;border:1px solid rgba(124,77,255,.7);border-radius:7px;background:#7c4dff29;color:#fff;cursor:pointer;font-size:12px}.workbench-chat-generate-patch:hover:not(:disabled){background:#7c4dff42}.workbench-chat-generate-patch:disabled{opacity:.55;cursor:not-allowed}.workbench-chat-toolbar.compact{grid-template-columns:minmax(0,auto) 1fr auto}.workbench-chat-route-summary{min-width:0;overflow:hidden;text-overflow:ellipsis;white-space:nowrap;color:var(--workbench-muted);font-size:11px}.workbench-chat-toolbar.atomek-input{height:auto;min-height:40px;display:grid;grid-template-columns:auto minmax(0,1fr) auto;align-items:center;gap:8px;padding:6px 8px 8px}.workbench-chat-toolbar.atomek-input .workbench-chat-route-summary{font-size:11px}.workbench-chat-toolbar-right{display:inline-flex;align-items:center;gap:6px}.workbench-chat-attach,.workbench-chat-target{position:relative}.workbench-chat-attach>summary,.workbench-chat-target>summary{list-style:none;cursor:pointer;display:inline-flex;align-items:center;height:30px;border:1px solid var(--border-subtle, #34343a);background:var(--bg-elevated, #2b2b2d);color:var(--text-secondary, #c9c9d0);border-radius:8px;user-select:none}.workbench-chat-attach>summary{width:30px;justify-content:center}.workbench-chat-attach>summary::-webkit-details-marker,.workbench-chat-target>summary::-webkit-details-marker{display:none}.workbench-chat-attach>summary:hover,.workbench-chat-target>summary:hover{background:#3a3a3d;color:#fff;border-color:#7c4dff73}.workbench-chat-attach[open]>summary,.workbench-chat-target[open]>summary{background:#7c4dff2e;border-color:#7c4dff8c;color:#fff}.workbench-chat-attach-menu,.workbench-chat-target-menu{position:absolute;z-index:30;min-width:200px;padding:4px;background:var(--bg-elevated, #1f1f22);border:1px solid var(--border-subtle, #34343a);border-radius:8px;box-shadow:0 12px 28px #00000073;display:flex;flex-direction:column;gap:2px}.workbench-chat-attach-menu{bottom:calc(100% + 6px);left:0}.workbench-chat-target-menu{bottom:calc(100% + 6px);right:0;min-width:240px}.workbench-chat-attach-menu button,.workbench-chat-target-menu button{height:30px;border:0;background:transparent;color:var(--text-primary, #e6e6ea);text-align:left;padding:0 10px;border-radius:6px;font-size:12px;cursor:pointer;display:flex;align-items:center;gap:6px}.workbench-chat-attach-menu button:hover:not(:disabled),.workbench-chat-target-menu button:hover:not(:disabled){background:#7c4dff29}.workbench-chat-attach-menu button:disabled,.workbench-chat-target-menu button:disabled{opacity:.45;cursor:not-allowed}.workbench-chat-target-menu button.active{background:#7c4dff3d;color:#fff}.workbench-chat-target-menu button small{margin-left:auto;font-size:10px;color:#f6c177}.workbench-chat-mic{width:30px;height:30px;display:inline-grid;place-items:center;border:1px solid var(--border-subtle, #34343a);background:var(--bg-elevated, #2b2b2d);color:var(--text-secondary, #c9c9d0);border-radius:8px;cursor:pointer}.workbench-chat-mic:hover:not(:disabled){background:#7c4dff29;color:#fff;border-color:#7c4dff73}.workbench-chat-mic:disabled{opacity:.45;cursor:not-allowed}.workbench-chat-target>summary{padding:0 8px 0 10px;gap:4px;font-size:12px;font-weight:600;max-width:220px}.workbench-chat-target-label{max-width:180px;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}.workbench-chat-target-row-label{flex:1;min-width:0;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}.workbench-chat-recording{display:inline-flex;align-items:center;gap:8px;padding:0 6px;height:30px;min-width:0}.workbench-chat-recording-cancel{width:26px;height:26px;display:inline-grid;place-items:center;border:0;border-radius:6px;background:#f443362e;color:#f44336;cursor:pointer}.workbench-chat-recording-cancel:hover{background:#f4433652}.workbench-chat-recording-wave{display:inline-flex;align-items:center;gap:2px;height:18px}.workbench-chat-recording-wave span{width:3px;border-radius:2px;background:#7c4dff;animation:workbenchChatWave .7s ease-in-out infinite alternate;height:30%}.workbench-chat-recording-wave span:nth-child(3n){background:#d85cae}@keyframes workbenchChatWave{0%{height:20%}to{height:90%}}.workbench-chat-recording-dot{width:8px;height:8px;border-radius:50%;background:#f44336;animation:workbenchChatPulse 1s ease-in-out infinite}@keyframes workbenchChatPulse{0%,to{opacity:1}50%{opacity:.35}}.workbench-chat-recording-time{font-family:ui-monospace,SFMono-Regular,Menlo,Monaco,Consolas,monospace;font-size:11px;color:var(--workbench-muted)}.workbench-manual-check-panel{display:flex;flex-direction:column;gap:12px;padding:12px}.workbench-manual-check-head,.workbench-check-actions,.workbench-check-add-row{display:flex;align-items:center;gap:8px}.workbench-manual-check-head small{color:var(--workbench-muted, #8b949e)}.workbench-manual-check-grid{display:grid;grid-template-columns:minmax(260px,1fr) minmax(320px,1.3fr);gap:12px}.workbench-manual-check-grid section,.workbench-check-results{display:flex;flex-direction:column;gap:8px}.workbench-check-command-list{display:flex;flex-direction:column;gap:6px;max-height:160px;overflow:auto}.workbench-check-command-list button{display:flex;justify-content:space-between;gap:8px;text-align:left}.workbench-check-command-list button.active{outline:1px solid var(--workbench-accent, #6aa6ff)}.workbench-check-add-row input,.workbench-manual-check-grid textarea,.workbench-manual-check-grid select{width:100%}.workbench-check-status{border-radius:999px;padding:2px 8px;font-size:11px;text-transform:uppercase}.workbench-check-status.pending{background:#d299222e;color:#d29922}.workbench-check-status.failed{background:#f851492e;color:#ff7b72}.workbench-check-status.passed{background:#3fb9502e;color:#56d364}.workbench-check-results article{border:1px solid var(--workbench-border, #30363d);border-radius:8px;padding:8px}.workbench-check-results pre{max-height:160px;overflow:auto;white-space:pre-wrap}.workbench-workbench{grid-template-columns:48px minmax(0,var(--workbench-primary-width, 300px)) minmax(0,1fr) minmax(320px,var(--workbench-secondary-width, 520px))}.workbench-workbench.no-primary{grid-template-columns:48px 0 minmax(0,1fr) minmax(320px,var(--workbench-secondary-width, 460px))}.workbench-workbench.no-secondary{grid-template-columns:48px minmax(0,var(--workbench-primary-width, 300px)) minmax(0,1fr) 0}.workbench-workbench.no-primary.no-secondary{grid-template-columns:48px 0 minmax(0,1fr) 0}.workbench-primary-region,.workbench-main,.workbench-secondary,.workbench-chat-wrap,.workbench-chat-transcript,.workbench-chat-message,.workbench-rich-body,.workbench-rich-markdown,.workbench-editor-stack,.workbench-editor-pane{min-width:0}.workbench-secondary{overflow:hidden}.workbench-secondary-tab-group{min-width:0;overflow:hidden}.workbench-secondary-tab{min-width:0;overflow:hidden;text-overflow:ellipsis}.workbench-chat-threadbar{min-width:0}.workbench-chat-threadbar select{width:100%}.workbench-chat-message{overflow:hidden;max-width:100%;box-sizing:border-box}.workbench-chat-wrap{grid-template-columns:minmax(0,1fr)}.workbench-chat-attachments{flex-wrap:wrap!important}.workbench-chat-chip{max-width:100%}.workbench-chat-threadbar{display:flex;flex-wrap:wrap;align-items:center;gap:6px}.workbench-chat-threadbar>select{flex:1 1 150px;min-width:0;width:auto}.workbench-chat-threadbar>.workbench-chat-route-summary{flex:1 1 100%}.workbench-chat-threadbar>button{flex:0 0 auto}.workbench-chat-threadbar>details{flex:0 0 auto}.workbench-chat-threadmenu{position:relative;flex:0 0 auto}.workbench-chat-threadmenu>summary{list-style:none;height:26px;width:30px;display:grid;place-items:center;border:1px solid var(--workbench-border-strong);border-radius:5px;background:#252526;color:var(--workbench-text);cursor:pointer}.workbench-chat-threadmenu>summary::-webkit-details-marker{display:none}.workbench-chat-threadmenu>summary::marker{content:\"\"}.workbench-chat-threadmenu[open]>summary{border-color:var(--workbench-blue, #4a9eff)}.workbench-chat-threadmenu-pop{position:absolute;right:0;top:calc(100% + 4px);z-index:20;display:flex;flex-direction:column;min-width:132px;padding:4px;border:1px solid var(--workbench-border-strong);border-radius:6px;background:#2a2d2e;box-shadow:0 8px 22px #00000059}.workbench-chat-threadmenu-pop button{text-align:left;height:28px;padding:0 10px;border:0;border-radius:4px;background:transparent;color:var(--workbench-text);font-size:12px;cursor:pointer}.workbench-chat-threadmenu-pop button:hover:not(:disabled){background:#ffffff14}.workbench-chat-threadmenu-pop button.danger:hover:not(:disabled){background:#d85cae2e;color:#f2d2ec}.workbench-chat-threadmenu-pop button:disabled{opacity:.5;cursor:not-allowed}.workbench-chat-message-actions button{width:30px;padding:0;gap:0;justify-content:center}.workbench-chat-tip{display:flex;align-items:center;flex-wrap:wrap;gap:3px 7px}.workbench-chat-tip>em{margin-left:auto}.workbench-chat-tip-sep{color:var(--workbench-muted);opacity:.55}.workbench-rich-body,.workbench-rich-markdown{max-width:100%;overflow-wrap:anywhere}.workbench-rich-markdown table{display:block;max-width:100%;overflow:auto;border-collapse:collapse;margin:10px 0 12px;white-space:normal}.workbench-rich-markdown th,.workbench-rich-markdown td{border:1px solid rgba(255,255,255,.1);padding:6px 8px;vertical-align:top;min-width:120px}.workbench-rich-markdown th{color:#f0f0f3;background:#ffffff0d}.workbench-rich-code,.workbench-rich-code pre{max-width:100%}.workbench-file-row,.workbench-tree-row,.workbench-folder-row{min-width:0}.workbench-file-row svg,.workbench-tree-row svg,.workbench-folder-row svg{flex:0 0 auto}.workbench-row-text{min-width:0;display:grid;gap:1px;line-height:1.2}.workbench-row-name,.workbench-row-detail{min-width:0;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}.workbench-row-detail{color:var(--workbench-muted);font-size:10px}.workbench-agent-dock{min-width:0;min-height:0;height:100%;display:flex;flex-direction:column;background:var(--workbench-side);overflow:hidden}.workbench-agent-dock-scroll{min-height:0;overflow:auto;padding:12px}.workbench-agent-run{margin:10px 0 12px;border:1px solid var(--border-subtle, #34343a);border-radius:var(--tytus-radius-sm, 7px);background:#0000002e;overflow:hidden}.workbench-agent-run header{min-height:34px;display:flex;align-items:center;justify-content:space-between;gap:8px;padding:7px 8px;border-bottom:1px solid rgba(255,255,255,.07);background:#ffffff09}.workbench-agent-run header>div{min-width:0;display:grid;gap:2px}.workbench-agent-run header strong,.workbench-agent-run header span{min-width:0;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}.workbench-agent-run header span{color:var(--workbench-muted);font-size:11px;text-transform:uppercase;letter-spacing:.04em}.workbench-run-history{display:grid;gap:6px;margin:6px 0 12px}.workbench-run-history-row{display:grid;grid-template-columns:minmax(0,1fr) auto;gap:8px;align-items:center;border:1px solid rgba(255,255,255,.07);background:#ffffff06;border-radius:var(--tytus-radius-sm, 7px);padding:7px 8px}.workbench-run-history-row>div{min-width:0;display:grid;gap:2px}.workbench-run-history-row strong,.workbench-run-history-row span,.workbench-run-history-row small{min-width:0;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}.workbench-run-history-row span,.workbench-run-history-row small{color:var(--workbench-muted);font-size:11px}.workbench-run-history-row small{font-family:var(--font-mono, ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace)}.workbench-agent-primary-action{border-color:#7c4dff66!important;background:linear-gradient(135deg,#7c4dff33,#d85cae1f)!important;color:#f5efff!important;font-weight:650}.workbench-computer-actions{display:grid;grid-template-columns:repeat(auto-fit,minmax(118px,1fr));gap:6px}.workbench-computer-card,.workbench-computer-context-card,.workbench-computer-hero,.workbench-computer-job-prompt,.workbench-computer-job-log{max-width:100%;box-sizing:border-box}.workbench-computer-job-log{max-height:280px;overflow:auto;white-space:pre-wrap;overflow-wrap:anywhere}@media(max-width:1180px){.workbench-workbench{grid-template-columns:48px minmax(0,min(260px,var(--workbench-primary-width, 260px))) minmax(0,1fr) minmax(300px,min(42vw,var(--workbench-secondary-width, 420px)))}}@media(max-width:980px){.workbench-workbench{grid-template-columns:48px minmax(0,240px) minmax(0,1fr) 0}.workbench-secondary{display:none}.workbench-statusbar{grid-column:3 / 4}}@media(max-width:760px){.workbench-workbench,.workbench-workbench.no-primary,.workbench-workbench.no-secondary,.workbench-workbench.no-primary.no-secondary{grid-template-columns:48px 0 minmax(0,1fr) 0}.workbench-primary-region,.workbench-secondary{display:none}.workbench-statusbar{grid-column:3 / 4}}.workbench-control-home{padding:clamp(20px,3vw,34px);overflow:auto}.workbench-control-hero-main{border:1px solid rgba(124,77,255,.28);background:linear-gradient(135deg,#7c4dff29,#569cd614,#00000014);border-radius:14px;padding:clamp(18px,3vw,28px);margin-bottom:18px;max-width:1180px}.workbench-control-kicker{color:#b79cff;font-size:11px;font-weight:800;letter-spacing:.14em;text-transform:uppercase;margin-bottom:8px}.workbench-control-hero-main h1{font-size:clamp(34px,5vw,58px);line-height:.98;margin:0 0 12px;font-weight:800;color:#f3f3f4}.workbench-control-hero-main p{max-width:860px;color:var(--workbench-muted);font-size:15px;line-height:1.55;margin:0 0 18px}.workbench-control-goal-row{display:grid;grid-template-columns:minmax(0,1fr) 190px;gap:12px;align-items:stretch}.workbench-control-goal-row textarea{width:100%;box-sizing:border-box;resize:vertical;min-height:96px;color:var(--workbench-text);background:#00000038;border:1px solid rgba(255,255,255,.12);border-radius:10px;padding:12px;font:inherit}.workbench-control-hero-actions{display:grid;gap:8px}.workbench-team-preset-strip{display:grid;grid-template-columns:repeat(4,minmax(0,1fr));gap:8px;margin-top:12px}.workbench-team-preset-card{min-width:0;border:1px solid rgba(255,255,255,.08);background:#ffffff09;color:var(--workbench-text);border-radius:11px;padding:10px;text-align:left;cursor:pointer}.workbench-team-preset-card.active{border-color:#7c4dffb8;background:linear-gradient(135deg,#7c4dff33,#569cd614);box-shadow:inset 0 0 0 1px #7c4dff2e}.workbench-team-preset-card>div{display:flex;align-items:center;justify-content:space-between;gap:8px}.workbench-team-preset-card strong,.workbench-team-preset-card span,.workbench-team-preset-card p,.workbench-team-preset-card small{display:block;min-width:0;overflow:hidden;text-overflow:ellipsis}.workbench-team-preset-card strong{white-space:nowrap}.workbench-team-preset-card span{color:#9af0b4;font-size:10px;font-weight:800;text-transform:uppercase}.workbench-team-preset-card.partial span,.workbench-team-preset-card.needs-setup span{color:#ffd48a}.workbench-team-preset-card p,.workbench-team-preset-card small{color:var(--workbench-muted);font-size:11px;line-height:1.35;margin:7px 0 0}.workbench-team-preset-card small{white-space:nowrap;font-size:10px}.workbench-control-success{margin-top:10px;color:#9af0b4;font-size:12px}.workbench-control-success code{color:#d7ba7d}.workbench-control-grid{display:grid;grid-template-columns:repeat(2,minmax(260px,1fr));gap:12px;max-width:1180px}.workbench-control-card{border:1px solid var(--workbench-border-strong);background:#252526eb;border-radius:12px;padding:14px;min-width:0}.workbench-control-card.wide{grid-column:1 / -1}.workbench-control-card header{display:flex;align-items:center;justify-content:space-between;gap:12px;margin-bottom:12px}.workbench-control-card header strong{color:var(--workbench-text)}.workbench-control-card header span{color:var(--workbench-muted);font-size:12px}.workbench-control-metrics{display:grid;grid-template-columns:repeat(5,minmax(110px,1fr));gap:8px}.workbench-control-metric{border:1px solid rgba(255,255,255,.08);background:#ffffff09;border-radius:10px;padding:10px;min-width:0}.workbench-control-metric strong{display:block;color:#f3f3f4;font-size:24px}.workbench-control-metric span,.workbench-control-metric em{display:block;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}.workbench-control-metric span{color:var(--workbench-text);font-size:12px;font-weight:700}.workbench-control-metric em{color:var(--workbench-muted);font-size:11px;font-style:normal}.workbench-agent-brand-grid{display:grid;grid-template-columns:repeat(4,minmax(0,1fr));gap:10px}.workbench-agent-brand-card{position:relative;overflow:hidden;border:1px solid rgba(124,77,255,.22);background:linear-gradient(135deg,#7c4dff1f,#569cd60d);border-radius:12px;padding:12px;min-height:116px}.workbench-agent-brand-card:after{content:\"\";position:absolute;inset:auto -24px -34px auto;width:92px;height:92px;border-radius:999px;background:#7c4dff29}.workbench-agent-brand-card>div{display:flex;align-items:center;justify-content:space-between;gap:10px;position:relative;z-index:1}.workbench-agent-brand-card strong,.workbench-agent-brand-card span,.workbench-agent-brand-card b,.workbench-agent-brand-card p{position:relative;z-index:1}.workbench-agent-brand-card strong{color:#f3f3f4}.workbench-agent-brand-card span{color:#9af0b4;font-size:10px;font-weight:800;text-transform:uppercase;letter-spacing:.06em}.workbench-agent-brand-card.available-when-installed span,.workbench-agent-brand-card.not-allocated span,.workbench-agent-brand-card.needs-setup span{color:#ffd48a}.workbench-agent-brand-card.missing span{color:#ff9e9e}.workbench-agent-brand-card b{display:block;color:#fff;font-size:30px;line-height:1;margin-top:14px}.workbench-agent-brand-card p{color:var(--workbench-muted);font-size:11px;line-height:1.35;margin:8px 0 0}.workbench-fabric-flow{display:grid;grid-template-columns:repeat(4,minmax(0,1fr));gap:10px}.workbench-fabric-node{border:1px solid rgba(86,156,214,.22);background:#569cd60e;border-radius:12px;padding:12px;min-width:0}.workbench-fabric-node>span{display:inline-flex;align-items:center;justify-content:center;width:22px;height:22px;border-radius:999px;color:#fff;background:linear-gradient(135deg,#7c4dff,#569cd6);font-size:11px;font-weight:800;margin-bottom:10px}.workbench-fabric-node strong,.workbench-fabric-node em,.workbench-fabric-node p{display:block;min-width:0}.workbench-fabric-node strong{color:#f3f3f4}.workbench-fabric-node em{color:#9af0b4;font-size:10px;font-style:normal;font-weight:800;text-transform:uppercase;letter-spacing:.06em;margin-top:3px}.workbench-fabric-node.needs-setup em,.workbench-fabric-node.optional em{color:#ffd48a}.workbench-fabric-node p{color:var(--workbench-muted);font-size:11px;line-height:1.4;margin:8px 0 0}.workbench-fabric-actions{display:flex;flex-wrap:wrap;gap:8px;margin-top:12px}.workbench-control-preset{width:100%;text-align:left;color:var(--workbench-text);background:#ffffff0a;border:1px solid rgba(255,255,255,.08);border-radius:8px;padding:10px;margin-bottom:8px;cursor:pointer}.workbench-control-preset small{display:block;color:var(--workbench-muted);font-size:10px;margin-top:4px;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}.workbench-control-preset:hover{border-color:#7c4dff73;background:#7c4dff1f}.workbench-control-recent{display:grid;gap:6px;margin-top:12px}.workbench-control-recent span{color:var(--workbench-muted);font-size:11px;font-weight:800;letter-spacing:.08em;text-transform:uppercase}.workbench-control-recent button{color:#8cbcff;background:transparent;border:0;padding:0;text-align:left;cursor:pointer}.workbench-control-resource-list{display:grid;grid-template-columns:repeat(2,minmax(0,1fr));gap:8px}.workbench-control-resource{display:grid;grid-template-columns:minmax(0,1fr) auto;gap:8px;align-items:center;border:1px solid rgba(255,255,255,.08);background:#ffffff06;border-radius:8px;padding:9px 10px;min-width:0}.workbench-control-resource strong,.workbench-control-resource span{display:block;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}.workbench-control-resource div>span{color:var(--workbench-muted);font-size:11px;margin-top:2px}.workbench-control-loop{display:grid;grid-template-columns:repeat(5,minmax(120px,1fr));gap:8px;list-style:none;padding:0;margin:0}.workbench-control-loop li{border:1px solid rgba(86,156,214,.2);background:#569cd60f;border-radius:10px;padding:10px}.workbench-control-loop b,.workbench-control-loop span{display:block}.workbench-control-loop span{color:var(--workbench-muted);font-size:11px;margin-top:4px}.workbench-control-loop code{color:#d7ba7d}.workbench-task-graph.home{grid-template-columns:repeat(3,minmax(0,1fr));margin-bottom:0}@media(max-width:1100px){.workbench-control-goal-row,.workbench-control-grid,.workbench-agent-brand-grid,.workbench-fabric-flow,.workbench-team-preset-strip,.workbench-control-resource-list,.workbench-task-graph.home{grid-template-columns:1fr}.workbench-control-metrics,.workbench-control-loop{grid-template-columns:repeat(2,minmax(0,1fr))}}@media(max-width:720px){.workbench-control-metrics,.workbench-control-loop{grid-template-columns:1fr}}.workbench-button-subtle.danger{border-color:#ff6b6b59;color:#ffb4b4;background:#ff6b6b14}.workbench-button-subtle.danger:hover{background:#ff6b6b24}.workbench-task-card.active{border-color:#7c4dffb3;background:#7c4dff29;box-shadow:inset 2px 0 #7c4dffd9}.workbench-agent-run-actions{display:flex!important;flex-direction:row!important;align-items:center;justify-content:flex-end;gap:6px!important;flex-shrink:0}.workbench-workbench{font-family:var(--atomek-font-body);background:var(--atomek-void);color:var(--atomek-bone)}.workbench-activity-bar{background:#0b0c0d;border-right-color:#f0ead81a}.workbench-activity-brand{width:48px;height:46px;display:grid;place-items:center;margin-bottom:4px;border:0;border-bottom:1px solid rgba(240,234,216,.08);background:transparent;cursor:pointer;color:var(--atomek-bone)}.workbench-activity-brand:hover{background:#b4ff0014}.workbench-activity-brand img,.workbench-brand-lockup img{user-select:none;pointer-events:none;object-fit:contain}.workbench-activity-button{color:#f0ead88f}.workbench-activity-button:hover,.workbench-activity-button.active{color:var(--atomek-bone)}.workbench-activity-button.active:before{background:var(--atomek-acid);width:3px}.workbench-sidebar,.workbench-secondary,.workbench-tabs,.workbench-secondary-tabs,.workbench-command-center,.workbench-breadcrumb{background:var(--workbench-panel-2);border-color:#f0ead81a}.workbench-tab.active,.workbench-file-row.active,.workbench-tree-row.active{background:#c4ff001a;color:var(--atomek-bone)}.workbench-input,.workbench-control-goal-row textarea,.workbench-settings-label select,.workbench-settings-label input{background:#0d0e10;border-color:#f0ead829;color:var(--atomek-bone);border-radius:4px}.workbench-input:focus,.workbench-control-goal-row textarea:focus,.workbench-settings-label select:focus,.workbench-settings-label input:focus{border-color:var(--atomek-acid);box-shadow:0 0 0 1px #c4ff0026}.workbench-button-primary,.workbench-button-blue{background:var(--atomek-acid);color:var(--atomek-void);border-radius:4px;font-weight:800;letter-spacing:.02em;text-transform:uppercase}.workbench-button-primary:hover,.workbench-button-blue:hover{filter:brightness(.94);background:var(--atomek-acid)}.workbench-button-subtle{background:#f0ead80a;color:var(--atomek-bone);border-color:#f0ead824;border-radius:4px}.workbench-button-subtle:hover{background:#c4ff0017;border-color:#c4ff0073}.workbench-statusbar{background:var(--atomek-acid);color:var(--atomek-void);font-family:var(--atomek-font-mono);font-weight:700}.workbench-welcome{background:var(--atomek-void);color:var(--atomek-bone)}.workbench-brand-lockup{display:flex;align-items:center;gap:clamp(12px,2vw,22px);margin-bottom:16px}.workbench-brand-wordmark{display:block;width:min(340px,54vw);max-height:76px;object-fit:contain;object-position:left center}.workbench-control-home{background:radial-gradient(circle at 86% 4%,rgba(196,255,0,.1),transparent 30%),var(--atomek-void)}.workbench-control-hero-main{position:relative;overflow:hidden;border-color:#c4ff0047;background:#0f1012f5;border-radius:8px;box-shadow:inset 0 0 0 1px #f0ead80d}.workbench-control-hero-main:after{content:\"\";position:absolute;right:-80px;top:-92px;width:230px;height:230px;border:1px solid rgba(196,255,0,.18);border-radius:999px;pointer-events:none}.workbench-control-kicker{color:var(--atomek-acid);font-family:var(--atomek-font-mono);letter-spacing:.16em}.workbench-control-hero-main h1,.workbench-welcome h1{font-family:var(--atomek-font-display);text-transform:uppercase;color:var(--atomek-bone);letter-spacing:-.04em}.workbench-control-hero-main p{color:#f0ead8b8}.workbench-team-preset-card,.workbench-control-card,.workbench-control-metric,.workbench-control-resource,.workbench-control-loop li,.workbench-control-preset,.workbench-fabric-node,.workbench-agent-brand-card{border-color:#f0ead81f;background:#f0ead809;border-radius:6px}.workbench-team-preset-card.active,.workbench-agent-brand-card.ready,.workbench-fabric-node.ready{border-color:#c4ff008c;background:#c4ff0013;box-shadow:inset 0 0 0 1px #c4ff0014}.workbench-agent-brand-card:after{background:#c4ff001f}.workbench-agent-brand-card span,.workbench-fabric-node em,.workbench-team-preset-card span,.workbench-control-success{color:var(--atomek-acid)}.workbench-agent-brand-card.available-when-installed span,.workbench-agent-brand-card.not-allocated span,.workbench-agent-brand-card.needs-setup span,.workbench-fabric-node.needs-setup em,.workbench-fabric-node.optional em,.workbench-team-preset-card.partial span,.workbench-team-preset-card.needs-setup span{color:var(--atomek-hazard)}.workbench-agent-brand-card.missing span{color:var(--atomek-fission)}.workbench-fabric-node>span{background:var(--atomek-acid);color:var(--atomek-void)}.workbench-control-card header strong,.workbench-control-metric strong,.workbench-agent-brand-card strong,.workbench-agent-brand-card b,.workbench-fabric-node strong{color:var(--atomek-bone)}.workbench-control-card header span,.workbench-control-metric em,.workbench-agent-brand-card p,.workbench-fabric-node p,.workbench-control-loop span,.workbench-team-preset-card p,.workbench-team-preset-card small{color:#f0ead894}.workbench-control-preset:hover{border-color:#c4ff0073;background:#c4ff0014}.workbench-start-link,.workbench-control-recent button{color:var(--atomek-acid)}.workbench-secondary-tab.active,.workbench-panel-tabs span:first-child{border-bottom-color:var(--atomek-acid);color:var(--atomek-bone)}.workbench-chat-message.user{border-color:#c4ff004d}.workbench-chat-message-actions button.regen{color:var(--atomek-acid);border-color:#c4ff0042}@media(max-width:720px){.workbench-brand-lockup{align-items:flex-start}.workbench-brand-lockup>img{width:54px!important;height:54px!important}.workbench-brand-wordmark{width:min(260px,70vw)}}.workbench-doc-grid{display:grid;grid-template-columns:repeat(5,minmax(0,1fr));gap:10px}.workbench-doc-card{min-width:0;text-align:left;color:var(--workbench-text);border:1px solid rgba(124,77,255,.24);background:linear-gradient(135deg,#7c4dff1c,#569cd60e);border-radius:12px;padding:12px;cursor:pointer}.workbench-doc-card:hover{border-color:#7c4dff9e;background:linear-gradient(135deg,#7c4dff30,#569cd614)}.workbench-doc-card strong,.workbench-doc-card p,.workbench-doc-card small{display:block;min-width:0;overflow:hidden;text-overflow:ellipsis}.workbench-doc-card strong{color:var(--atomek-acid, #d8ff6f);white-space:nowrap}.workbench-doc-card p{min-height:48px;color:var(--workbench-muted);font-size:11px;line-height:1.35;margin:8px 0 10px}.workbench-doc-card small{color:#f0ead899;font-size:10px;white-space:nowrap}@media(max-width:1180px){.workbench-doc-grid{grid-template-columns:repeat(2,minmax(0,1fr))}}@media(max-width:720px){.workbench-doc-grid{grid-template-columns:1fr}}.monaco-aria-container{position:absolute;left:-999em}::-ms-clear{display:none}.monaco-editor .editor-widget input{color:inherit}.monaco-editor{position:relative;overflow:visible;-webkit-text-size-adjust:100%;color:var(--vscode-editor-foreground);background-color:var(--vscode-editor-background);overflow-wrap:initial}.monaco-editor-background{background-color:var(--vscode-editor-background)}.monaco-editor .rangeHighlight{background-color:var(--vscode-editor-rangeHighlightBackground);box-sizing:border-box;border:1px solid var(--vscode-editor-rangeHighlightBorder)}.monaco-editor.hc-black .rangeHighlight,.monaco-editor.hc-light .rangeHighlight{border-style:dotted}.monaco-editor .symbolHighlight{background-color:var(--vscode-editor-symbolHighlightBackground);box-sizing:border-box;border:1px solid var(--vscode-editor-symbolHighlightBorder)}.monaco-editor.hc-black .symbolHighlight,.monaco-editor.hc-light .symbolHighlight{border-style:dotted}.monaco-editor .editorCanvas{position:absolute;width:100%;height:100%;z-index:0;pointer-events:none}.monaco-editor .overflow-guard{position:relative;overflow:hidden}.monaco-editor .view-overlays{position:absolute;top:0}.monaco-editor .view-overlays>div,.monaco-editor .margin-view-overlays>div{position:absolute;width:100%}.monaco-editor .squiggly-error{border-bottom:4px double var(--vscode-editorError-border)}.monaco-editor .squiggly-error:before{display:block;content:\"\";width:100%;height:100%;background:var(--vscode-editorError-background)}.monaco-editor .squiggly-warning{border-bottom:4px double var(--vscode-editorWarning-border)}.monaco-editor .squiggly-warning:before{display:block;content:\"\";width:100%;height:100%;background:var(--vscode-editorWarning-background)}.monaco-editor .squiggly-info{border-bottom:4px double var(--vscode-editorInfo-border)}.monaco-editor .squiggly-info:before{display:block;content:\"\";width:100%;height:100%;background:var(--vscode-editorInfo-background)}.monaco-editor .squiggly-hint{border-bottom:2px dotted var(--vscode-editorHint-border)}.monaco-editor.showUnused .squiggly-unnecessary{border-bottom:2px dashed var(--vscode-editorUnnecessaryCode-border)}.monaco-editor.showDeprecated .squiggly-inline-deprecated{text-decoration:line-through;text-decoration-color:var(--vscode-editor-foreground, inherit)}.monaco-scrollable-element>.scrollbar>.scra{cursor:pointer;font-size:11px!important}.monaco-scrollable-element>.visible{opacity:1;background:#0000;transition:opacity .1s linear;z-index:11}.monaco-scrollable-element>.invisible{opacity:0;pointer-events:none}.monaco-scrollable-element>.invisible.fade{transition:opacity .8s linear}.monaco-scrollable-element>.shadow{position:absolute;display:none}.monaco-scrollable-element>.shadow.top{display:block;top:0;left:3px;height:3px;width:100%;box-shadow:var(--vscode-scrollbar-shadow) 0 6px 6px -6px inset}.monaco-scrollable-element>.shadow.left{display:block;top:3px;left:0;height:100%;width:3px;box-shadow:var(--vscode-scrollbar-shadow) 6px 0 6px -6px inset}.monaco-scrollable-element>.shadow.top-left-corner{display:block;top:0;left:0;height:3px;width:3px}.monaco-scrollable-element>.shadow.top.left{box-shadow:var(--vscode-scrollbar-shadow) 6px 0 6px -6px inset}.monaco-scrollable-element>.scrollbar{background:var(--vscode-scrollbar-background)}.monaco-scrollable-element>.scrollbar>.slider{background:var(--vscode-scrollbarSlider-background)}.monaco-scrollable-element>.scrollbar>.slider:hover{background:var(--vscode-scrollbarSlider-hoverBackground)}.monaco-scrollable-element>.scrollbar>.slider.active{background:var(--vscode-scrollbarSlider-activeBackground)}.monaco-editor .blockDecorations-container{position:absolute;top:0;pointer-events:none}.monaco-editor .blockDecorations-block{position:absolute;box-sizing:border-box}.monaco-editor .view-overlays .current-line,.monaco-editor .margin-view-overlays .current-line{display:block;position:absolute;left:0;top:0;box-sizing:border-box;height:100%}.monaco-editor .margin-view-overlays .current-line.current-line-margin.current-line-margin-both{border-right:0}.monaco-editor .lines-content .cdr{position:absolute;height:100%}.monaco-editor .glyph-margin{position:absolute;top:0}.monaco-editor .glyph-margin-widgets .cgmr{position:absolute;display:flex;align-items:center;justify-content:center}.monaco-editor .glyph-margin-widgets .cgmr.codicon-modifier-spin:before{position:absolute;top:50%;left:50%;transform:translate(-50%,-50%)}.monaco-editor .lines-content .core-guide{position:absolute;box-sizing:border-box;height:100%}.monaco-editor .margin-view-overlays .line-numbers{bottom:0;font-variant-numeric:tabular-nums;position:absolute;text-align:right;display:inline-block;vertical-align:middle;box-sizing:border-box;cursor:default}.monaco-editor .relative-current-line-number{text-align:left;display:inline-block;width:100%}.monaco-editor .margin-view-overlays .line-numbers.lh-odd{margin-top:1px}.monaco-editor .line-numbers{color:var(--vscode-editorLineNumber-foreground)}.monaco-editor .line-numbers.active-line-number{color:var(--vscode-editorLineNumber-activeForeground)}.monaco-mouse-cursor-text{cursor:text}.mtkcontrol{color:#fff!important;background:#960000!important}.mtkoverflow{background-color:var(--vscode-button-background, var(--vscode-editor-background));color:var(--vscode-button-foreground, var(--vscode-editor-foreground));border-width:1px;border-style:solid;border-color:var(--vscode-contrastBorder);border-radius:2px;padding:4px;cursor:pointer}.mtkoverflow:hover{background-color:var(--vscode-button-hoverBackground)}.monaco-editor.no-user-select .lines-content,.monaco-editor.no-user-select .view-line,.monaco-editor.no-user-select .view-lines{user-select:none;-webkit-user-select:none}.monaco-editor.mac .lines-content:hover,.monaco-editor.mac .view-line:hover,.monaco-editor.mac .view-lines:hover{user-select:text;-webkit-user-select:text;-ms-user-select:text}.monaco-editor.enable-user-select{user-select:initial;-webkit-user-select:initial}.monaco-editor .view-lines{white-space:nowrap}.monaco-editor .view-line{box-sizing:border-box;position:absolute;width:100%}.monaco-editor .lines-content>.view-lines>.view-line>span{top:0;bottom:0;position:absolute}.monaco-editor .mtkw{color:var(--vscode-editorWhitespace-foreground)!important}.monaco-editor .mtkz{display:inline-block;color:var(--vscode-editorWhitespace-foreground)!important}.monaco-editor .lines-decorations{position:absolute;top:0;background:#fff}.monaco-editor .margin-view-overlays .cldr{position:absolute;height:100%}.monaco-editor .margin{background-color:var(--vscode-editorGutter-background)}.monaco-editor .margin-view-overlays .cmdr{position:absolute;left:0;width:100%;height:100%}.monaco-editor .minimap.slider-mouseover .minimap-slider{opacity:0;transition:opacity .1s linear}.monaco-editor .minimap.slider-mouseover:hover .minimap-slider,.monaco-editor .minimap.slider-mouseover .minimap-slider.active{opacity:1}.monaco-editor .minimap-slider .minimap-slider-horizontal{background:var(--vscode-minimapSlider-background)}.monaco-editor .minimap-slider:hover .minimap-slider-horizontal{background:var(--vscode-minimapSlider-hoverBackground)}.monaco-editor .minimap-slider.active .minimap-slider-horizontal{background:var(--vscode-minimapSlider-activeBackground)}.monaco-editor .minimap-shadow-visible{box-shadow:var(--vscode-scrollbar-shadow) -6px 0 6px -6px inset}.monaco-editor .minimap-shadow-hidden{position:absolute;width:0}.monaco-editor .minimap-shadow-visible{position:absolute;left:-6px;width:6px;pointer-events:none}.monaco-editor.no-minimap-shadow .minimap-shadow-visible{position:absolute;left:-1px;width:1px}.minimap.minimap-autohide-mouseover,.minimap.minimap-autohide-scroll{opacity:0;transition:opacity .5s}.minimap.minimap-autohide-scroll{pointer-events:none}.minimap.minimap-autohide-mouseover:hover,.minimap.minimap-autohide-scroll.active{opacity:1;pointer-events:auto}.monaco-editor .minimap{z-index:5}.monaco-editor .overlayWidgets{position:absolute;top:0;left:0}.monaco-editor .view-ruler{position:absolute;top:0;box-shadow:1px 0 0 0 var(--vscode-editorRuler-foreground) inset}.monaco-editor .scroll-decoration{position:absolute;top:0;left:0;height:6px;box-shadow:var(--vscode-scrollbar-shadow) 0 6px 6px -6px inset}.monaco-editor .lines-content .cslr{position:absolute}.monaco-editor .focused .selected-text{background-color:var(--vscode-editor-selectionBackground)}.monaco-editor .selected-text{background-color:var(--vscode-editor-inactiveSelectionBackground)}.monaco-editor .top-left-radius{border-top-left-radius:3px}.monaco-editor .bottom-left-radius{border-bottom-left-radius:3px}.monaco-editor .top-right-radius{border-top-right-radius:3px}.monaco-editor .bottom-right-radius{border-bottom-right-radius:3px}.monaco-editor.hc-black .top-left-radius{border-top-left-radius:0}.monaco-editor.hc-black .bottom-left-radius{border-bottom-left-radius:0}.monaco-editor.hc-black .top-right-radius{border-top-right-radius:0}.monaco-editor.hc-black .bottom-right-radius{border-bottom-right-radius:0}.monaco-editor.hc-light .top-left-radius{border-top-left-radius:0}.monaco-editor.hc-light .bottom-left-radius{border-bottom-left-radius:0}.monaco-editor.hc-light .top-right-radius{border-top-right-radius:0}.monaco-editor.hc-light .bottom-right-radius{border-bottom-right-radius:0}.monaco-editor .cursors-layer{position:absolute;top:0}.monaco-editor .cursors-layer>.cursor{position:absolute;overflow:hidden;box-sizing:border-box}.monaco-editor .cursors-layer.cursor-smooth-caret-animation>.cursor{transition:all 80ms}.monaco-editor .cursors-layer.cursor-block-outline-style>.cursor{background:transparent!important;border-style:solid;border-width:1px}.monaco-editor .cursors-layer.cursor-underline-style>.cursor{border-bottom-width:2px;border-bottom-style:solid;background:transparent!important}.monaco-editor .cursors-layer.cursor-underline-thin-style>.cursor{border-bottom-width:1px;border-bottom-style:solid;background:transparent!important}@keyframes monaco-cursor-smooth{0%,20%{opacity:1}60%,to{opacity:0}}@keyframes monaco-cursor-phase{0%,20%{opacity:1}90%,to{opacity:0}}@keyframes monaco-cursor-expand{0%,20%{transform:scaleY(1)}80%,to{transform:scaleY(0)}}.cursor-smooth{animation:monaco-cursor-smooth .5s ease-in-out 0s 20 alternate}.cursor-phase{animation:monaco-cursor-phase .5s ease-in-out 0s 20 alternate}.cursor-expand>.cursor{animation:monaco-cursor-expand .5s ease-in-out 0s 20 alternate}.monaco-editor .mwh{position:absolute;color:var(--vscode-editorWhitespace-foreground)!important}.monaco-editor .monaco-decoration-css-rule-extractor{visibility:hidden;pointer-events:none}.monaco-editor .inputarea{min-width:0;min-height:0;margin:0;padding:0;position:absolute;outline:none!important;resize:none;border:none;overflow:hidden;color:transparent;background-color:transparent;z-index:-10}.monaco-editor .inputarea.ime-input{z-index:10;caret-color:var(--vscode-editorCursor-foreground);color:var(--vscode-editor-foreground)}.monaco-editor .native-edit-context{margin:0;padding:0;position:absolute;overflow-y:scroll;scrollbar-width:none;z-index:-10;white-space:pre-wrap}.monaco-editor .ime-text-area{min-width:0;min-height:0;margin:0;padding:0;position:absolute;outline:none!important;resize:none;border:none;overflow:hidden;color:transparent;background-color:transparent;z-index:-10}.monaco-editor .edit-context-composition-none{background-color:transparent;border-bottom:none}.monaco-editor :not(.hc-black,.hc-light) .edit-context-composition-secondary{border-bottom:1px solid var(--vscode-editor-compositionBorder)}.monaco-editor :not(.hc-black,.hc-light) .edit-context-composition-primary{border-bottom:2px solid var(--vscode-editor-compositionBorder)}.monaco-editor :is(.hc-black,.hc-light) .edit-context-composition-secondary{border:1px solid var(--vscode-editor-compositionBorder)}.monaco-editor :is(.hc-black,.hc-light) .edit-context-composition-primary{border:2px solid var(--vscode-editor-compositionBorder)}.monaco-editor .margin-view-overlays .gpu-mark{position:absolute;top:0;bottom:0;left:0;width:100%;display:inline-block;border-left:solid 2px var(--vscode-editorWarning-foreground);opacity:.2;transition:background-color .1s linear}.monaco-editor .margin-view-overlays .gpu-mark:hover{background-color:var(--vscode-editorWarning-foreground)}.monaco-select-box{width:100%;cursor:pointer;border-radius:2px}.monaco-select-box-dropdown-container{font-size:13px;font-weight:400;text-transform:none}.monaco-action-bar .action-item.select-container{cursor:default}.monaco-action-bar .action-item .monaco-select-box{cursor:pointer;min-width:100px;min-height:18px;padding:2px 23px 2px 8px}.mac .monaco-action-bar .action-item .monaco-select-box{font-size:11px;border-radius:3px;min-height:24px}.monaco-list{position:relative;height:100%;width:100%;white-space:nowrap}.monaco-list.mouse-support{user-select:none;-webkit-user-select:none}.monaco-list>.monaco-scrollable-element{height:100%}.monaco-list-rows{position:relative;width:100%;height:100%}.monaco-list.horizontal-scrolling .monaco-list-rows{width:auto;min-width:100%}.monaco-list-row{position:absolute;box-sizing:border-box;overflow:hidden;width:100%}.monaco-list.mouse-support .monaco-list-row{cursor:pointer;touch-action:none}.monaco-list .monaco-scrollable-element>.scrollbar.vertical,.monaco-pane-view>.monaco-split-view2.vertical>.monaco-scrollable-element>.scrollbar.vertical{z-index:14}.monaco-list-row.scrolling{display:none!important}.monaco-list.element-focused,.monaco-list.selection-single,.monaco-list.selection-multiple{outline:0!important}.monaco-list-type-filter-message{position:absolute;box-sizing:border-box;width:100%;height:100%;top:0;left:0;padding:40px 1em 1em;text-align:center;white-space:normal;opacity:.7;pointer-events:none}.monaco-list-type-filter-message:empty{display:none}.monaco-drag-image{display:inline-block;padding:1px 7px;border-radius:10px;font-size:12px;position:absolute;z-index:1000;background-color:var(--vscode-list-activeSelectionBackground);color:var(--vscode-list-activeSelectionForeground);outline:1px solid var(--vscode-list-focusOutline);outline-offset:-1px;max-width:120px;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}.monaco-select-box-dropdown-padding{--dropdown-padding-top: 1px;--dropdown-padding-bottom: 1px}.hc-black .monaco-select-box-dropdown-padding,.hc-light .monaco-select-box-dropdown-padding{--dropdown-padding-top: 3px;--dropdown-padding-bottom: 4px}.monaco-select-box-dropdown-container{display:none;box-sizing:border-box}.monaco-select-box-dropdown-container>.select-box-details-pane>.select-box-description-markdown *{margin:0}.monaco-select-box-dropdown-container>.select-box-details-pane>.select-box-description-markdown a:focus{outline:1px solid -webkit-focus-ring-color;outline-offset:-1px}.monaco-select-box-dropdown-container>.select-box-details-pane>.select-box-description-markdown code{line-height:15px;font-family:var(--monaco-monospace-font)}.monaco-select-box-dropdown-container.visible{display:flex;flex-direction:column;text-align:left;width:1px;overflow:hidden;border-bottom-left-radius:3px;border-bottom-right-radius:3px}.monaco-select-box-dropdown-container>.select-box-dropdown-list-container{flex:0 0 auto;align-self:flex-start;padding-top:var(--dropdown-padding-top);padding-bottom:var(--dropdown-padding-bottom);padding-left:1px;padding-right:1px;width:100%;overflow:hidden;box-sizing:border-box}.monaco-select-box-dropdown-container>.select-box-details-pane{padding:5px}.hc-black .monaco-select-box-dropdown-container>.select-box-dropdown-list-container{padding-top:var(--dropdown-padding-top);padding-bottom:var(--dropdown-padding-bottom)}.monaco-select-box-dropdown-container>.select-box-dropdown-list-container .monaco-list .monaco-list-row{cursor:pointer}.monaco-select-box-dropdown-container>.select-box-dropdown-list-container .monaco-list .monaco-list-row>.option-text{text-overflow:ellipsis;overflow:hidden;padding-left:3.5px;white-space:nowrap;float:left}.monaco-select-box-dropdown-container>.select-box-dropdown-list-container .monaco-list .monaco-list-row>.option-detail{text-overflow:ellipsis;overflow:hidden;padding-left:3.5px;white-space:nowrap;float:left;opacity:.7}.monaco-select-box-dropdown-container>.select-box-dropdown-list-container .monaco-list .monaco-list-row>.option-decorator-right{text-overflow:ellipsis;overflow:hidden;padding-right:10px;white-space:nowrap;float:right}.monaco-select-box-dropdown-container>.select-box-dropdown-list-container .monaco-list .monaco-list-row>.visually-hidden{position:absolute;left:-10000px;top:auto;width:1px;height:1px;overflow:hidden}.monaco-select-box-dropdown-container>.select-box-dropdown-container-width-control{flex:1 1 auto;align-self:flex-start;opacity:0}.monaco-select-box-dropdown-container>.select-box-dropdown-container-width-control>.width-control-div{overflow:hidden;max-height:0px}.monaco-select-box-dropdown-container>.select-box-dropdown-container-width-control>.width-control-div>.option-text-width-control{padding-left:4px;padding-right:8px;white-space:nowrap}.monaco-action-bar{white-space:nowrap;height:100%}.monaco-action-bar .actions-container{display:flex;margin:0 auto;padding:0;height:100%;width:100%;align-items:center}.monaco-action-bar.vertical .actions-container{display:inline-block}.monaco-action-bar .action-item{display:block;align-items:center;justify-content:center;cursor:pointer;position:relative}.monaco-action-bar .action-item.disabled{cursor:default}.monaco-action-bar .action-item .icon,.monaco-action-bar .action-item .codicon{display:block}.monaco-action-bar .action-item .codicon{display:flex;align-items:center;width:16px;height:16px}.monaco-action-bar .action-label{display:flex;font-size:11px;padding:3px;border-radius:5px}.monaco-action-bar .action-item.disabled .action-label:not(.icon),.monaco-action-bar .action-item.disabled .action-label:not(.icon):before,.monaco-action-bar .action-item.disabled .action-label:not(.icon):hover{color:var(--vscode-disabledForeground)}.monaco-action-bar .action-item.disabled .action-label.icon,.monaco-action-bar .action-item.disabled .action-label.icon:before,.monaco-action-bar .action-item.disabled .action-label.icon:hover{opacity:.6}.monaco-action-bar.vertical{text-align:left}.monaco-action-bar.vertical .action-item{display:block}.monaco-action-bar.vertical .action-label.separator{display:block;border-bottom:1px solid var(--vscode-disabledForeground);padding-top:1px;margin-left:.8em;margin-right:.8em}.monaco-action-bar .action-item .action-label.separator{width:1px;height:16px;margin:5px 4px!important;cursor:default;min-width:1px;padding:0;background-color:var(--vscode-disabledForeground)}.secondary-actions .monaco-action-bar .action-label{margin-left:6px}.monaco-action-bar .action-item.select-container{overflow:hidden;flex:1;max-width:170px;min-width:60px;display:flex;align-items:center;justify-content:center;margin-right:10px}.monaco-action-bar .action-item.action-dropdown-item{display:flex}.monaco-action-bar .action-item.action-dropdown-item>.action-dropdown-item-separator{display:flex;align-items:center;cursor:default}.monaco-action-bar .action-item.action-dropdown-item>.action-dropdown-item-separator>div{width:1px}.monaco-diff-editor .diff-review{position:absolute}.monaco-component.diff-review{user-select:none;-webkit-user-select:none;z-index:99;.diff-review-line-number{text-align:right;display:inline-block;color:var(--vscode-editorLineNumber-foreground)}.diff-review-summary{padding-left:10px}.diff-review-shadow{position:absolute;box-shadow:var(--vscode-scrollbar-shadow) 0 -6px 6px -6px inset}.diff-review-row{white-space:pre}.diff-review-table{display:table;min-width:100%}.diff-review-row{display:table-row;width:100%}.diff-review-spacer{display:inline-block;width:10px;vertical-align:middle}.diff-review-spacer>.codicon{font-size:9px!important}.diff-review-actions{display:inline-block;position:absolute;right:10px;top:2px;z-index:100}.diff-review-actions .action-label{width:16px;height:16px;margin:2px 0}.revertButton{cursor:pointer}.action-label{background:var(--vscode-editorActionList-background)}}:root{--vscode-sash-size: 4px;--vscode-sash-hover-size: 4px}.monaco-sash{position:absolute;z-index:35;touch-action:none}.monaco-sash.disabled{pointer-events:none}.monaco-sash.mac.vertical{cursor:col-resize}.monaco-sash.vertical.minimum{cursor:e-resize}.monaco-sash.vertical.maximum{cursor:w-resize}.monaco-sash.mac.horizontal{cursor:row-resize}.monaco-sash.horizontal.minimum{cursor:s-resize}.monaco-sash.horizontal.maximum{cursor:n-resize}.monaco-sash.disabled{cursor:default!important;pointer-events:none!important}.monaco-sash.vertical{cursor:ew-resize;top:0;width:var(--vscode-sash-size);height:100%}.monaco-sash.horizontal{cursor:ns-resize;left:0;width:100%;height:var(--vscode-sash-size)}.monaco-sash:not(.disabled)>.orthogonal-drag-handle{content:\" \";height:calc(var(--vscode-sash-size) * 2);width:calc(var(--vscode-sash-size) * 2);z-index:100;display:block;cursor:all-scroll;position:absolute}.monaco-sash.horizontal.orthogonal-edge-north:not(.disabled)>.orthogonal-drag-handle.start,.monaco-sash.horizontal.orthogonal-edge-south:not(.disabled)>.orthogonal-drag-handle.end{cursor:nwse-resize}.monaco-sash.horizontal.orthogonal-edge-north:not(.disabled)>.orthogonal-drag-handle.end,.monaco-sash.horizontal.orthogonal-edge-south:not(.disabled)>.orthogonal-drag-handle.start{cursor:nesw-resize}.monaco-sash.vertical>.orthogonal-drag-handle.start{left:calc(var(--vscode-sash-size) * -.5);top:calc(var(--vscode-sash-size) * -1)}.monaco-sash.vertical>.orthogonal-drag-handle.end{left:calc(var(--vscode-sash-size) * -.5);bottom:calc(var(--vscode-sash-size) * -1)}.monaco-sash.horizontal>.orthogonal-drag-handle.start{top:calc(var(--vscode-sash-size) * -.5);left:calc(var(--vscode-sash-size) * -1)}.monaco-sash.horizontal>.orthogonal-drag-handle.end{top:calc(var(--vscode-sash-size) * -.5);right:calc(var(--vscode-sash-size) * -1)}.monaco-sash:before{content:\"\";pointer-events:none;position:absolute;width:100%;height:100%;background:transparent}.monaco-enable-motion .monaco-sash:before{transition:background-color .1s ease-out}.monaco-sash.hover:before,.monaco-sash.active:before{background:var(--vscode-sash-hoverBorder)}.monaco-sash.vertical:before{width:var(--vscode-sash-hover-size);left:calc(50% - (var(--vscode-sash-hover-size) / 2))}.monaco-sash.horizontal:before{height:var(--vscode-sash-hover-size);top:calc(50% - (var(--vscode-sash-hover-size) / 2))}.pointer-events-disabled{pointer-events:none!important}.monaco-sash.debug{background:#0ff}.monaco-sash.debug.disabled{background:#0ff3}.monaco-sash.debug:not(.disabled)>.orthogonal-drag-handle{background:red}.monaco-dropdown{height:100%;padding:0}.monaco-dropdown>.dropdown-label{cursor:pointer;height:100%;display:flex;align-items:center;justify-content:center}.monaco-dropdown>.dropdown-label>.action-label.disabled{cursor:default}.monaco-dropdown-with-primary{display:flex!important;flex-direction:row;border-radius:5px}.monaco-dropdown-with-primary>.action-container>.action-label{margin-right:0}.monaco-dropdown-with-primary>.dropdown-action-container>.monaco-dropdown>.dropdown-label .codicon[class*=codicon-]{font-size:12px;padding-left:0;padding-right:0;line-height:16px;margin-left:-3px}.monaco-dropdown-with-primary>.dropdown-action-container>.monaco-dropdown>.dropdown-label>.action-label{display:block;background-size:16px;background-position:center center;background-repeat:no-repeat}.monaco-toolbar{height:100%}.monaco-toolbar .toolbar-toggle-more{display:inline-block;padding:0}.monaco-toolbar.responsive{.monaco-action-bar>.actions-container>.action-item{flex-shrink:1;min-width:20px}}.monaco-action-bar .action-item.menu-entry .action-label.icon{width:16px;height:16px;background-repeat:no-repeat;background-position:50%;background-size:16px}.monaco-action-bar .action-item.menu-entry.text-only .action-label{color:var(--vscode-descriptionForeground);overflow:hidden;border-radius:2px}.monaco-action-bar .action-item.menu-entry.text-only.use-comma:not(:last-of-type) .action-label:after{content:\", \"}.monaco-action-bar .action-item.menu-entry.text-only+.action-item:not(.text-only)>.monaco-dropdown .action-label{color:var(--vscode-descriptionForeground)}.monaco-dropdown-with-default{display:flex!important;flex-direction:row;border-radius:5px}.monaco-dropdown-with-default>.action-container>.action-label{margin-right:0}.monaco-dropdown-with-default>.action-container.menu-entry>.action-label.icon{width:16px;height:16px;background-repeat:no-repeat;background-position:50%;background-size:16px}.monaco-dropdown-with-default:hover{background-color:var(--vscode-toolbar-hoverBackground)}.monaco-dropdown-with-default>.dropdown-action-container>.monaco-dropdown>.dropdown-label .codicon[class*=codicon-]{font-size:12px;padding-left:0;padding-right:0;line-height:16px;margin-left:-3px}.monaco-dropdown-with-default>.dropdown-action-container>.monaco-dropdown>.dropdown-label>.action-label{display:block;background-size:16px;background-position:center center;background-repeat:no-repeat}.monaco-editor .diff-hidden-lines-widget{width:100%}.monaco-editor .diff-hidden-lines{height:0px;transform:translateY(-10px);font-size:13px;line-height:14px}.monaco-editor .diff-hidden-lines:not(.dragging) .top:hover,.monaco-editor .diff-hidden-lines:not(.dragging) .bottom:hover,.monaco-editor .diff-hidden-lines .top.dragging,.monaco-editor .diff-hidden-lines .bottom.dragging{background-color:var(--vscode-focusBorder)}.monaco-editor .diff-hidden-lines .top,.monaco-editor .diff-hidden-lines .bottom{transition:background-color .1s ease-out;height:4px;background-color:transparent;background-clip:padding-box;border-bottom:2px solid transparent;border-top:4px solid transparent}.monaco-editor.draggingUnchangedRegion.canMoveTop:not(.canMoveBottom) *,.monaco-editor .diff-hidden-lines .top.canMoveTop:not(.canMoveBottom),.monaco-editor .diff-hidden-lines .bottom.canMoveTop:not(.canMoveBottom){cursor:n-resize!important}.monaco-editor.draggingUnchangedRegion:not(.canMoveTop).canMoveBottom *,.monaco-editor .diff-hidden-lines .top:not(.canMoveTop).canMoveBottom,.monaco-editor .diff-hidden-lines .bottom:not(.canMoveTop).canMoveBottom{cursor:s-resize!important}.monaco-editor.draggingUnchangedRegion.canMoveTop.canMoveBottom *,.monaco-editor .diff-hidden-lines .top.canMoveTop.canMoveBottom,.monaco-editor .diff-hidden-lines .bottom.canMoveTop.canMoveBottom{cursor:ns-resize!important}.monaco-editor .diff-hidden-lines .top{transform:translateY(4px)}.monaco-editor .diff-hidden-lines .bottom{transform:translateY(-6px)}.monaco-editor .diff-unchanged-lines{background:var(--vscode-diffEditor-unchangedCodeBackground)}.monaco-editor .noModificationsOverlay{z-index:1;background:var(--vscode-editor-background);display:flex;justify-content:center;align-items:center}.monaco-editor .diff-hidden-lines .center{background:var(--vscode-diffEditor-unchangedRegionBackground);color:var(--vscode-diffEditor-unchangedRegionForeground);overflow:hidden;display:block;text-overflow:ellipsis;white-space:nowrap;height:24px;box-shadow:inset 0 -5px 5px -7px var(--vscode-diffEditor-unchangedRegionShadow),inset 0 5px 5px -7px var(--vscode-diffEditor-unchangedRegionShadow)}.monaco-editor .diff-hidden-lines .center span.codicon{vertical-align:middle}.monaco-editor .diff-hidden-lines .center a:hover .codicon{cursor:pointer;color:var(--vscode-editorLink-activeForeground)!important}.monaco-editor .diff-hidden-lines div.breadcrumb-item{cursor:pointer}.monaco-editor .diff-hidden-lines div.breadcrumb-item:hover{color:var(--vscode-editorLink-activeForeground)}.monaco-editor .movedOriginal,.monaco-editor .movedModified{border:2px solid var(--vscode-diffEditor-move-border)}.monaco-editor .movedOriginal.currentMove,.monaco-editor .movedModified.currentMove{border:2px solid var(--vscode-diffEditor-moveActive-border)}.monaco-diff-editor .moved-blocks-lines path.currentMove{stroke:var(--vscode-diffEditor-moveActive-border)}.monaco-diff-editor .moved-blocks-lines path{pointer-events:visiblestroke}.monaco-diff-editor .moved-blocks-lines .arrow{fill:var(--vscode-diffEditor-move-border)}.monaco-diff-editor .moved-blocks-lines .arrow.currentMove{fill:var(--vscode-diffEditor-moveActive-border)}.monaco-diff-editor .moved-blocks-lines .arrow-rectangle{fill:var(--vscode-editor-background)}.monaco-diff-editor .moved-blocks-lines{position:absolute;pointer-events:none}.monaco-diff-editor .moved-blocks-lines path{fill:none;stroke:var(--vscode-diffEditor-move-border);stroke-width:2}.monaco-editor .char-delete.diff-range-empty{margin-left:-1px;border-left:solid var(--vscode-diffEditor-removedTextBackground) 3px}.monaco-editor .char-insert.diff-range-empty{border-left:solid var(--vscode-diffEditor-insertedTextBackground) 3px}.monaco-editor .fold-unchanged{cursor:pointer}.monaco-diff-editor .diff-moved-code-block{display:flex;justify-content:flex-end;margin-top:-4px}.monaco-diff-editor .diff-moved-code-block .action-bar .action-label.codicon{width:12px;height:12px;font-size:12px}.monaco-diff-editor .diffOverview{z-index:9}.monaco-diff-editor .diffOverview .diffViewport{z-index:10}.monaco-diff-editor.vs .diffOverview{background:#00000008}.monaco-diff-editor.vs-dark .diffOverview{background:#ffffff03}.monaco-scrollable-element.modified-in-monaco-diff-editor.vs .scrollbar,.monaco-scrollable-element.modified-in-monaco-diff-editor.vs-dark .scrollbar{background:#0000}.monaco-scrollable-element.modified-in-monaco-diff-editor.hc-black .scrollbar,.monaco-scrollable-element.modified-in-monaco-diff-editor.hc-light .scrollbar{background:none}.monaco-scrollable-element.modified-in-monaco-diff-editor .slider{z-index:10}.modified-in-monaco-diff-editor .slider.active{background:#ababab66}.modified-in-monaco-diff-editor.hc-black .slider.active,.modified-in-monaco-diff-editor.hc-light .slider.active{background:none}.monaco-editor .insert-sign,.monaco-diff-editor .insert-sign,.monaco-editor .delete-sign,.monaco-diff-editor .delete-sign{font-size:11px!important;opacity:.7!important;display:flex!important;align-items:center}.monaco-editor.hc-black .insert-sign,.monaco-diff-editor.hc-black .insert-sign,.monaco-editor.hc-black .delete-sign,.monaco-diff-editor.hc-black .delete-sign,.monaco-editor.hc-light .insert-sign,.monaco-diff-editor.hc-light .insert-sign,.monaco-editor.hc-light .delete-sign,.monaco-diff-editor.hc-light .delete-sign{opacity:1}.monaco-editor .inline-deleted-margin-view-zone,.monaco-editor .inline-added-margin-view-zone{text-align:right}.monaco-editor .arrow-revert-change{z-index:10;position:absolute}.monaco-editor .arrow-revert-change:hover{cursor:pointer}.monaco-editor .view-zones .view-lines .view-line span{display:inline-block}.monaco-editor .margin-view-zones .lightbulb-glyph:hover{cursor:pointer}.monaco-editor .char-insert,.monaco-diff-editor .char-insert{background-color:var(--vscode-diffEditor-insertedTextBackground)}.monaco-editor .line-insert,.monaco-diff-editor .line-insert{background-color:var(--vscode-diffEditor-insertedLineBackground, var(--vscode-diffEditor-insertedTextBackground))}.monaco-editor .line-insert,.monaco-editor .char-insert{box-sizing:border-box;border:1px solid var(--vscode-diffEditor-insertedTextBorder)}.monaco-editor.hc-black .line-insert,.monaco-editor.hc-light .line-insert,.monaco-editor.hc-black .char-insert,.monaco-editor.hc-light .char-insert{border-style:dashed}.monaco-editor .line-delete,.monaco-editor .char-delete{box-sizing:border-box;border:1px solid var(--vscode-diffEditor-removedTextBorder)}.monaco-editor.hc-black .line-delete,.monaco-editor.hc-light .line-delete,.monaco-editor.hc-black .char-delete,.monaco-editor.hc-light .char-delete{border-style:dashed}.monaco-editor .inline-added-margin-view-zone,.monaco-editor .gutter-insert,.monaco-diff-editor .gutter-insert{background-color:var(--vscode-diffEditorGutter-insertedLineBackground, var(--vscode-diffEditor-insertedLineBackground), var(--vscode-diffEditor-insertedTextBackground))}.monaco-editor .char-delete,.monaco-diff-editor .char-delete,.monaco-editor .inline-deleted-text{background-color:var(--vscode-diffEditor-removedTextBackground)}.monaco-editor .inline-deleted-text{text-decoration:line-through}.monaco-editor .line-delete,.monaco-diff-editor .line-delete{background-color:var(--vscode-diffEditor-removedLineBackground, var(--vscode-diffEditor-removedTextBackground))}.monaco-editor .inline-deleted-margin-view-zone,.monaco-editor .gutter-delete,.monaco-diff-editor .gutter-delete{background-color:var(--vscode-diffEditorGutter-removedLineBackground, var(--vscode-diffEditor-removedLineBackground), var(--vscode-diffEditor-removedTextBackground))}.monaco-diff-editor.side-by-side .editor.modified{box-shadow:-6px 0 5px -5px var(--vscode-scrollbar-shadow);border-left:1px solid var(--vscode-diffEditor-border)}.monaco-diff-editor.side-by-side .editor.original{box-shadow:6px 0 5px -5px var(--vscode-scrollbar-shadow);border-right:1px solid var(--vscode-diffEditor-border)}.monaco-diff-editor .diffViewport{background:var(--vscode-scrollbarSlider-background)}.monaco-diff-editor .diffViewport:hover{background:var(--vscode-scrollbarSlider-hoverBackground)}.monaco-diff-editor .diffViewport:active{background:var(--vscode-scrollbarSlider-activeBackground)}.monaco-editor .diagonal-fill{background-image:linear-gradient(-45deg,var(--vscode-diffEditor-diagonalFill) 12.5%,#0000 12.5%,#0000 50%,var(--vscode-diffEditor-diagonalFill) 50%,var(--vscode-diffEditor-diagonalFill) 62.5%,#0000 62.5%,#0000 100%);background-size:8px 8px}.monaco-diff-editor .gutter{position:relative;overflow:hidden;flex-shrink:0;flex-grow:0;>div{position:absolute}.gutterItem{opacity:0;transition:opacity .7s;&.showAlways{opacity:1;transition:none}&.noTransition{transition:none}}&:hover .gutterItem{opacity:1;transition:opacity .1s ease-in-out}.gutterItem{.background{position:absolute;height:100%;left:50%;width:1px;border-left:2px var(--vscode-menu-separatorBackground) solid}.buttons{position:absolute;width:100%;display:flex;justify-content:center;align-items:center;.monaco-toolbar{height:fit-content;.monaco-action-bar{line-height:1;.actions-container{width:fit-content;border-radius:4px;background:var(--vscode-editorGutter-itemBackground);.action-item{&:hover{background:var(--vscode-toolbar-hoverBackground)}.action-label{color:var(--vscode-editorGutter-itemGlyphForeground);padding:1px 2px}}}}}}}}.monaco-diff-editor .diff-hidden-lines-compact{display:flex;height:11px;.line-left,.line-right{height:1px;border-top:1px solid;border-color:var(--vscode-editorCodeLens-foreground);opacity:.5;margin:auto;width:100%}.line-left{width:20px}.text{color:var(--vscode-editorCodeLens-foreground);text-wrap:nowrap;font-size:11px;line-height:11px;margin:0 4px}}.monaco-editor .line-delete-selectable{user-select:text!important;-webkit-user-select:text!important;z-index:1!important}.line-delete-selectable .view-line{user-select:text!important;-webkit-user-select:text!important}.monaco-editor .selection-anchor{background-color:#007acc;width:2px!important}.monaco-editor .bracket-match{box-sizing:border-box;background-color:var(--vscode-editorBracketMatch-background);border:1px solid var(--vscode-editorBracketMatch-border)}.inline-editor-progress-decoration{display:inline-block;width:1em;height:1em}.inline-progress-widget{display:flex!important;justify-content:center;align-items:center}.inline-progress-widget .icon{font-size:80%!important}.inline-progress-widget:hover .icon{font-size:90%!important;animation:none}.inline-progress-widget:hover .icon:before{content:var(--vscode-icon-x-content);font-family:var(--vscode-icon-x-font-family)}.monaco-editor .monaco-editor-overlaymessage{padding-bottom:8px;z-index:10000}.monaco-editor .monaco-editor-overlaymessage.below{padding-bottom:0;padding-top:8px;z-index:10000}@keyframes fadeIn{0%{opacity:0}to{opacity:1}}.monaco-editor .monaco-editor-overlaymessage.fadeIn{animation:fadeIn .15s ease-out}@keyframes fadeOut{0%{opacity:1}to{opacity:0}}.monaco-editor .monaco-editor-overlaymessage.fadeOut{animation:fadeOut .1s ease-out}.monaco-editor .monaco-editor-overlaymessage .message{padding:2px 4px;color:var(--vscode-editorHoverWidget-foreground);background-color:var(--vscode-editorHoverWidget-background);border:1px solid var(--vscode-inputValidation-infoBorder);border-radius:3px}.monaco-editor .monaco-editor-overlaymessage .message p{margin-block:0px}.monaco-editor .monaco-editor-overlaymessage .message a{color:var(--vscode-textLink-foreground)}.monaco-editor .monaco-editor-overlaymessage .message a:hover{color:var(--vscode-textLink-activeForeground)}.monaco-editor.hc-black .monaco-editor-overlaymessage .message,.monaco-editor.hc-light .monaco-editor-overlaymessage .message{border-width:2px}.monaco-editor .monaco-editor-overlaymessage .anchor{width:0!important;height:0!important;border-color:transparent;border-style:solid;z-index:1000;border-width:8px;position:absolute;left:2px}.monaco-editor .monaco-editor-overlaymessage .anchor.top{border-bottom-color:var(--vscode-inputValidation-infoBorder)}.monaco-editor .monaco-editor-overlaymessage .anchor.below{border-top-color:var(--vscode-inputValidation-infoBorder)}.monaco-editor .monaco-editor-overlaymessage:not(.below) .anchor.top,.monaco-editor .monaco-editor-overlaymessage.below .anchor.below{display:none}.monaco-editor .monaco-editor-overlaymessage.below .anchor.top{display:inherit;top:-8px}.monaco-text-button{box-sizing:border-box;display:flex;width:100%;padding:4px;border-radius:2px;text-align:center;cursor:pointer;justify-content:center;align-items:center;border:1px solid var(--vscode-button-border, transparent);line-height:18px}.monaco-text-button:focus{outline-offset:2px!important}.monaco-text-button:hover{text-decoration:none!important}.monaco-button.disabled:focus,.monaco-button.disabled{opacity:.4!important;cursor:default}.monaco-text-button .codicon{margin:0 .2em;color:inherit!important}.monaco-text-button.monaco-text-button-with-short-label{flex-direction:row;flex-wrap:wrap;padding:0 4px;overflow:hidden;height:28px}.monaco-text-button.monaco-text-button-with-short-label>.monaco-button-label{flex-basis:100%}.monaco-text-button.monaco-text-button-with-short-label>.monaco-button-label-short{flex-grow:1;width:0;overflow:hidden}.monaco-text-button.monaco-text-button-with-short-label>.monaco-button-label,.monaco-text-button.monaco-text-button-with-short-label>.monaco-button-label-short{display:flex;justify-content:center;align-items:center;font-weight:400;font-style:inherit;padding:4px 0}.monaco-button-dropdown{display:flex;cursor:pointer}.monaco-button-dropdown.disabled{cursor:default}.monaco-button-dropdown>.monaco-button:focus{outline-offset:-1px!important}.monaco-button-dropdown.disabled>.monaco-button.disabled,.monaco-button-dropdown.disabled>.monaco-button.disabled:focus,.monaco-button-dropdown.disabled>.monaco-button-dropdown-separator{opacity:.4!important}.monaco-button-dropdown>.monaco-button.monaco-text-button{border-right-width:0!important}.monaco-button-dropdown .monaco-button-dropdown-separator{padding:4px 0;cursor:default}.monaco-button-dropdown .monaco-button-dropdown-separator>div{height:100%;width:1px}.monaco-button-dropdown>.monaco-button.monaco-dropdown-button{border:1px solid var(--vscode-button-border, transparent);border-left-width:0!important;border-radius:0 2px 2px 0;display:flex;align-items:center}.monaco-button-dropdown>.monaco-button.monaco-text-button{border-radius:2px 0 0 2px}.monaco-description-button{display:flex;flex-direction:column;align-items:center;margin:4px 5px}.monaco-description-button .monaco-button-description{font-style:italic;font-size:11px;padding:4px 20px}.monaco-description-button .monaco-button-label,.monaco-description-button .monaco-button-description{display:flex;justify-content:center;align-items:center}.monaco-description-button .monaco-button-label>.codicon,.monaco-description-button .monaco-button-description>.codicon{margin:0 .2em;color:inherit!important}.monaco-button.default-colors,.monaco-button-dropdown.default-colors>.monaco-button{color:var(--vscode-button-foreground);background-color:var(--vscode-button-background)}.monaco-button.default-colors:hover,.monaco-button-dropdown.default-colors>.monaco-button:hover{background-color:var(--vscode-button-hoverBackground)}.monaco-button.default-colors.secondary,.monaco-button-dropdown.default-colors>.monaco-button.secondary{color:var(--vscode-button-secondaryForeground);background-color:var(--vscode-button-secondaryBackground)}.monaco-button.default-colors.secondary:hover,.monaco-button-dropdown.default-colors>.monaco-button.secondary:hover{background-color:var(--vscode-button-secondaryHoverBackground)}.monaco-button-dropdown.default-colors .monaco-button-dropdown-separator{background-color:var(--vscode-button-background);border-top:1px solid var(--vscode-button-border);border-bottom:1px solid var(--vscode-button-border)}.monaco-button-dropdown.default-colors .monaco-button.secondary+.monaco-button-dropdown-separator{background-color:var(--vscode-button-secondaryBackground)}.monaco-button-dropdown.default-colors .monaco-button-dropdown-separator>div{background-color:var(--vscode-button-separator)}.action-widget{font-size:13px;min-width:100px;max-width:80vw;z-index:40;display:block;width:100%;border:1px solid var(--vscode-menu-border)!important;border-radius:5px;background-color:var(--vscode-menu-background);color:var(--vscode-menu-foreground);padding:4px;box-shadow:0 2px 8px var(--vscode-widget-shadow)}.context-view-block{position:fixed;cursor:initial;left:0;top:0;width:100%;height:100%;z-index:-1}.context-view-pointerBlock{position:fixed;cursor:initial;left:0;top:0;width:100%;height:100%;z-index:2}.action-widget .monaco-list{user-select:none;-webkit-user-select:none;border:none!important;border-width:0!important}.action-widget .monaco-list:focus:before{outline:0!important}.action-widget .monaco-list .monaco-scrollable-element{overflow:visible}.action-widget .monaco-list .monaco-list-row{padding:0 4px;white-space:nowrap;cursor:pointer;touch-action:none;width:100%;border-radius:3px}.action-widget .monaco-list .monaco-list-row.action.focused:not(.option-disabled){background-color:var(--vscode-list-activeSelectionBackground)!important;color:var(--vscode-list-activeSelectionForeground);outline:1px solid var(--vscode-menu-selectionBorder, transparent);outline-offset:-1px}.action-widget .monaco-list-row.group-header{color:var(--vscode-descriptionForeground)!important;font-weight:600;font-size:13px}.action-widget .monaco-list-row.group-header:not(:first-of-type){margin-top:2px}.action-widget .monaco-scrollable-element .monaco-list-rows .monaco-list-row.separator{border-top:1px solid var(--vscode-editorHoverWidget-border);color:var(--vscode-descriptionForeground);font-size:12px;padding:0;margin:4px 0 0;cursor:default;user-select:none;border-radius:0}.action-widget .monaco-scrollable-element .monaco-list-rows .monaco-list-row.separator.focused{outline:0 solid;background-color:transparent;border-radius:0}.action-widget .monaco-list-row.separator:first-of-type{border-top:none;margin-top:0}.action-widget .monaco-list .group-header,.action-widget .monaco-list .option-disabled,.action-widget .monaco-list .option-disabled:before,.action-widget .monaco-list .option-disabled .focused,.action-widget .monaco-list .option-disabled .focused:before{cursor:default!important;-webkit-touch-callout:none;-webkit-user-select:none;user-select:none;background-color:transparent!important;outline:0 solid!important}.action-widget .monaco-list-row.action{display:flex;gap:4px;align-items:center}.action-widget .monaco-list-row.action.option-disabled,.action-widget .monaco-list:focus .monaco-list-row.focused.action.option-disabled,.action-widget .monaco-list-row.action.option-disabled .codicon,.action-widget .monaco-list:not(.drop-target):not(.dragging) .monaco-list-row:hover:not(.selected):not(.focused).option-disabled{color:var(--vscode-disabledForeground)}.action-widget .monaco-list-row.action:not(.option-disabled) .codicon{color:inherit}.action-widget .monaco-list-row.action .title{flex:1;overflow:hidden;text-overflow:ellipsis}.action-widget .monaco-list-row.action .monaco-keybinding>.monaco-keybinding-key{background-color:var(--vscode-keybindingLabel-background);color:var(--vscode-keybindingLabel-foreground);border-style:solid;border-width:1px;border-radius:3px;border-color:var(--vscode-keybindingLabel-border);border-bottom-color:var(--vscode-keybindingLabel-bottomBorder);box-shadow:inset 0 -1px 0 var(--vscode-widget-shadow)}.action-widget .action-widget-action-bar{background-color:var(--vscode-menu-background);border-top:1px solid var(--vscode-menu-border);margin-top:2px}.action-widget .action-widget-action-bar:before{display:block;content:\"\";width:100%}.action-widget .action-widget-action-bar .actions-container{padding:4px 8px 2px 24px}.action-widget-action-bar .action-label{color:var(--vscode-textLink-activeForeground);font-size:13px;line-height:22px;padding:0;pointer-events:all}.action-widget-action-bar .action-item{margin-right:16px;pointer-events:none}.action-widget-action-bar .action-label:hover{background-color:transparent!important}.monaco-action-bar .actions-container.highlight-toggled .action-label.checked{background:var(--vscode-actionBar-toggledBackground)!important}.action-widget .monaco-list .monaco-list-row .description{opacity:.7;margin-left:.5em}.monaco-keybinding{display:flex;align-items:center;line-height:10px}.monaco-keybinding>.monaco-keybinding-key{display:inline-block;border-style:solid;border-width:1px;border-radius:3px;vertical-align:middle;font-size:11px;padding:3px 5px;margin:0 2px}.monaco-keybinding>.monaco-keybinding-key:first-child{margin-left:0}.monaco-keybinding>.monaco-keybinding-key:last-child{margin-right:0}.monaco-keybinding>.monaco-keybinding-key-separator{display:inline-block}.monaco-keybinding>.monaco-keybinding-key-chord-separator{width:6px}.post-edit-widget{box-shadow:0 0 8px 2px var(--vscode-widget-shadow);border:1px solid var(--vscode-widget-border, transparent);border-radius:4px;color:var(--vscode-button-foreground);background-color:var(--vscode-button-background);overflow:hidden}.post-edit-widget .monaco-button{padding:2px;border:none;border-radius:0}.post-edit-widget .monaco-button:hover{background-color:var(--vscode-button-hoverBackground)!important}.post-edit-widget .monaco-button .codicon{margin:0}@font-face{font-family:codicon;font-display:block;src:url(data:font/ttf;base64,AAEAAAALAIAAAwAwR1NVQiCLJXoAAAE4AAAAVE9TLzI3UEsvAAABjAAAAGBjbWFwdCJY8AAACfwAAB5QZ2x5ZpdPvvsAACxYAAGRYGhlYWRYkqBSAAAA4AAAADZoaGVhAlYDLwAAALwAAAAkaG10eFs1/+YAAAHsAAAIEGxvY2EPPKwaAAAoTAAABAptYXhwAx0BiAAAARgAAAAgbmFtZZP7uU8AAb24AAAB+HBvc3RPbs8TAAG/sAAAHMQAAQAAASwAAAAAASz/+v/+AS4AAQAAAAAAAAAAAAAAAAAAAgQAAQAAAAEAAD/d1LtfDzz1AAsBLAAAAAB8JbCAAAAAAHwlsID/+v/8AS4BLQAAAAgAAgAAAAAAAAABAAACBAF8AA8AAAAAAAIAAAAKAAoAAAD/AAAAAAAAAAEAAAAKADAAPgACREZMVAAObGF0bgAaAAQAAAAAAAAAAQAAAAQAAAAAAAAAAQAAAAFsaWdhAAgAAAABAAAAAQAEAAQAAAABAAgAAQAGAAAAAQAAAAQBKwGQAAUAAAC+ANIAAAAqAL4A0gAAAJAADgBNAAACAAUDAAAAAAAAAAAAAAAAAAAAAAAAAAAAAFBmRWQAwOpg8QMBLAAAABsBRwAEAAAAAQAAAAAAAAAAAAAAAAACAAAAAAEsAAABLAAAASwAAAEsAAABLAAAASwAAAEsAAABLAAAASwAAAEsAAABLAAAASwAAAEsAAABLAAAASwAAAEsAAABLAAAASwAAAEsAAABLAAAASwAAAEsAAABLAAAASwAAAEsAAABLAAAASwAAAEsAAABLAAAASwAAAEsAAABLAAAASwAAAEsAAABLAAAASwAAAEsAAABLAAAASwAAAEsAAABLAAAASwAAAEsAAABLAAAASwAAAEsAAABLAAAASwAAAEsAAABLAAAASwAAAEsAAABLAAAASwAAAEsAAABLAAAASwAAAEsAAABLAAAASwAAAEsAAABLAAAASwAAAEsAAABLAAAASwAAAEs//8BLAAAASwAAAEsAAABLAAAASwAAAEsAAABLAAAASwAAAEsAAABLAAAASwAAAEsAAABLAAAASwAAAEsAAABLAAAASwAAAEsAAABLAAAASwAAAEsAAABLAAAASwAAAEs//8BLP//ASz//wEsAAABLAAAASz//wEs//8BLP//ASwAAAEsAAABLAAAASwAAAEsAAABLAAAASwAAAEsAAABLAAAASwAAAEs//8BLP//ASz//wEsAAABLAAAASwAAAEsAAABLAAAASwAAAEsAAABLAAAASwAAAEsAAABLAAAASwAAAEsAAABLAAAASz//AEsAAABLP//ASwAAAEsAAABLAAAASwAAAEsAAABLAAAASwAAAEsAAABLAAAASwAAAEsAAABLAAAASwAAAEsAAABLAAAASwAAAEsAAABLAAAASwAAAEsAAABLAAAASwAAAEsAAABLAAAASwAAAEsAAABLAAAASwAAAEsAAABLAAAASwAAAEsAAABLAAAASwAAAEsAAABLAAAASwAAAEsAAABIAAAASwAAAEsAAABLAAAASwAAAEsAAABLAAAASwAAAEsAAABLAAAASwAAAEsAAABLAAAASwAAAEsAAABLAAAASwAAAEsAAABLAAAASwAAAEsAAABLAAAASwAAAEsAAABLP//ASwAAAEsAAABLAAAASwAAAEsAAABLAAAASwAAAEsAAABLAAAASwAAAEsAAABLAAAASwAAAEsAAABIAAAASwAAAEsAAABLAAAASwAAAEsAAABLAAAASwAAAEsAAABLAAAASwAAAEsAAABLAAAASAAAAEsAAABLAAAASD/+gEsAAABLAAAASwAAAEsAAABLAAAASwAAAEsAAABLAAAASwAAAEsAAABLAAAASwAAAEsAAABLAAAASwAAAEsAAABLAAAASwAAAEsAAABLAAAASwAAAEsAAABLAAAASwAAAEsAAABLAAAASwAAAEsAAABLAAAASwAAAEsAAABLAAAASwAAAEsAAABLAAAASwAAAEsAAABLAAAASwAAAEsAAABLAAAASwAAAEsAAABLAAAASwAAAEsAAABLAAAASwAAAEsAAABLAAAASwAAAEsAAABLAAAASwAAAEsAAABLAAAASwAAAEsAAABLAAAASwAAAEsAAABLAAAASwAAAEsAAABLAAAASwAAAEsAAABLAAAASwAAAEgAAABLAAAASwAAAEsAAABLAAAASwAAAEsAAABLAAAASwAAAEsAAABLAAAASwAAAEsAAABLAAAASwAAAEsAAABLAAAASwAAAEsAAABLAAAASwAAAEsAAABLAAAASwAAAEsAAABLAAAASwAAAEsAAABLAAAASwAAAEsAAABLAAAASwAAAEsAAABLAAAASwAAAEsAAABLAAAASwAAAEsAAABIAAAASwAAAEsAAABLAAAASwAAAEsAAABLAAAASwAAAEsAAABLAAAASwAAAEsAAABLAAAASwAAAEsAAABLAAAASwAAAEsAAABLAAAASwAAAEsAAABLAAAASwAAAEsAAABLAAAASwAAAEsAAABLAAAASwAAAEsAAABIAAAASwAAAEsAAABIAAAASwAAAEsAAABLAAAASwAAAEsAAABLAAAASwAAAEsAAABLAAAASwAAAEsAAABLAAAASwAAAEsAAABLAAAASwAAAEsAAABLAAAASwAAAEsAAABLAAAASwAAAEsAAABLAAAASwAAAEsAAABLAAAASwAAAEsAAABLAAAASwAAAEsAAABLAAAASwAAAEs//8BLAAAASwAAAEsAAABLAAAASwAAAEsAAABLAAAASwAAAEsAAABLAAAASwAAAEsAAABLAAAASwAAAEsAAABLAAAASwAAAEsAAABLAAAASwAAAEsAAABLAAAASwAAAEsAAABLAAAASwAAAEsAAABLAAAASwAAAEsAAABLAAAASwAAAEsAAABLAAAASwAAAEsAAABLAAAASwAAAEsAAABLAAAASwAAAEsAAABLAAAASwAAAEs//8BLAAAASwAAAEsAAABLAAAASwAAAEsAAABLAAAASwAAAEsAAABLAAAASwAAAEsAAABLAAAASwAAAEsAAABLAAAASwAAAEs//8BLAAAASwAAAEsAAABLAAAASz//wEsAAABLAAAASwAAAEsAAABLAAAASwAAAEsAAABLAAAASwAAAEsAAABLAAAASwAAAEsAAABLAAAASwAAAEsAAABLAAAASwAAAEsAAABLAAAASwAAAEsAAABLAAAASwAAAEsAAABLAAAASwAAAEsAAABLAAAASwAAAEsAAABLAAAASwAAAEsAAABLAAAASwAAAEsAAABLAAAASwAAAEsAAABLAAAASwAAAEsAAABLAAAASwAAAEsAAABLAAAAAAABQAAAAMAAAAsAAAABAAABaQAAQAAAAAEngADAAEAAAAsAAMACgAABaQABARyAAAAEgAQAAMAAuqI6ozqx+rJ6wnrTuxx8QP//wAA6mDqiuqP6snqzOsL61DxAf//AAAAAAAAAAAAAAAAAAAAAAABABIAYgBmANYA1gFQAdYEGAAAAAMBHAF8AXcA1gFmAckBUwDKAToBqQBXAfkBlAGfAZ4AqgA7AV0AnQDzASgARgHHAI0AGAH0ALUAnwFzAUsBQQFCAd4A7ADBAN4B1QG2AKMBxQGvAPsBvAGwAb4BxAHAAbkA4QG1AcIAAgAFAAYACwAMAA0ADgAPABAAEQATABwAHgAfACAAcABxAHIAcwB2AHcAIwAkACUAJgAoACsAMAAxADIAMwA0ADUANwA4ADkAOgBBAD4AQgBDAEQARQBHAEgATABOAFAAVABoAGoAawBsAHsAfQB/AIIAhgCIAIkAigCLAIwAjgCPAJAAkQCSAJMAlQCWAJgAmQCeAKAApACoAKkArACtAK4ArwCwALEAsgC0ALYAuAC6ALsAvAC9AL4AwADDAMQAxQDGAMsAzADPANoA2wDfAOMA5wDoAOsA7QDuAO8A8AD3APgA+QD6APsA/AD9AQEBGQEdAR4BIAEjASQBJQEmASoBKwEwATIBMwE5ATsBPAE9AT8BRAFFAUgBSgFNAU4BVgCGAVoBWwFcAV4BXwFhAWIBZAFlAWoBawFsAW0BbgFvAXEBcgF0AXYBeQF6AX0AlwF/AYABgQGCAYMBiwGMAY0BjgGPAZMBmQGaAZsBnQGhAaMBpgGnAagBqgGrAbEBsgGzAbQBtwC1AbgBugG9Ab8BwQHDAcsBzAHWAdgB2gHcAd0B3wHgAeEB4gHjAecB6QHqAesB7gE9Ae8B8QHzAfoB+wH8ACUB/gICAgMAuAEfASEBIgB0AHUAhAA/AIUAeAG5AIMAhwCBAG8AKQAqATQApQCrAOkB6AABABkAegEYAUwBhgHGAVgA3AGYAZcBUAGsAVkBaABuAfAASQE2AKYA5AEpAUcBaQAvAVcBTwA8AD0AUQHIAewB5gHkAeUA0QGEAYcBRgCAAf8CAQIAAc4BzwHRAdIB0wHUAc0AEgBmAVIAtwH4AH4A9QEEAQMBAgBaAFkAWAAWAPYA0ADTAG0AfAGJAL8AewAXAOUA5gFVACEAIgEnABUB7QFDARcBBQEGAQwBCQELAQ4BDwESARUBFgEIAQcBygDxAWcAogAHAAgACQAKARQBDQERAB0A6gEvASwAQAAbABoAVgDUANUBkABVAZYBpQD0ATgB2QHbAE0BogDCAfUANgFUAT4BNwF1AGUBGwF+AaQAlwCUAa4BnADZANcA2AH3AfYASgGIAYUAZwDd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AADrcQAAAeIAAOtyAADrcgAAAeMAAOtzAADrcwAAAecAAOt0AADrdAAAAekAAOt1AADrdQAAAeoAAOt2AADrdgAAAesAAOt3AADrdwAAAe4AAOt4AADreAAAAT0AAOt5AADreQAAAe8AAOt6AADregAAAfEAAOt7AADrewAAAfMAAOt8AADrfAAAAfoAAOt9AADrfQAAAfsAAOt+AADrfgAAAfwAAOt/AADrfwAAACUAAOuAAADrgAAAAf4AAOuBAADrgQAAAgIAAOuCAADrggAAAgMAAOuDAADrgwAAALgAAOuEAADrhAAAAR8AAOuFAADrhQAAASEAAOuGAADrhgAAASIAAOuHAADrhwAAAHQAAOuIAADriAAAAHUAAOuJAADriQAAAIQAAOuKAADrigAAAD8AAOuLAADriwAAAIUAAOuMAADrjAAAAHgAAOuNAADrjQAAAbkAAOuOAADrjgAAAIMAAOuPAADrjwAAAIcAAOuQAADrkAAAAIEAAOuRAADrkQAAAG8AAOuSAADrkgAAACkAAOuTAADrkwAAACoAAOuUAADrlAAAATQAAOuVAADrlQAAAKUAAOuWAADrlgAAAKsAAOuXAADrlwAAAOkAAOuYAADrmAAAAegAAOuZAADrmQAAAAEAAOuaAADrmgAAABkAAOubAADrmwAAAHoAAOucAADrnAAAARgAAOudAADrnQAAAUwAAOueAADrngAAAYYAAOufAADrnwAAAcYAAOugAADroAAAAVgAAOuhAADroQAAANwAAOuiAADrogAAAZgAAOujAADrowAAAZcAAOukAADrpAAAAVAAAOulAADrpQAAAawAAOumAADrpgAAAVkAAOunAADrpwAAAWgAAOuoAADrqAAAAG4AAOupAADrqQAAAfAAAOuqAADrqgAAAEkAAOurAADrqwAAATYAAOusAADrrAAAAKYAAOutAADrrQAAAOQAAOuuAADrrgAAASkAAOuvAADrrwAAAUcAAOuwAADrsAAAAWkAAOuxAADrsQAAAC8AAOuyAADrsgAAAVcAAOuzAADrswAAAU8AAOu0AADrtAAAADwAAOu1AADrtQAAAD0AAOu2AADrtgAAAFEAAOu3AADrtwAAAcgAAOu4AADruAAAAewAAOu5AADruQAAAeYAAOu6AADrugAAAeQAAOu7AADruwAAAeUAAOu8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AQxjb2RpY29uUmVndWxhcmNvZGljb25jb2RpY29uVmVyc2lvbiAxLjE1Y29kaWNvblRoZSBpY29uIGZvbnQgZm9yIFZpc3VhbCBTdHVkaW8gQ29kZWh0dHA6Ly9mb250ZWxsby5jb20AYwBvAGQAaQBjAG8AbgBSAGUAZwB1AGwAYQByAGMAbwBkAGkAYwBvAG4AYwBvAGQAaQBjAG8AbgBWAGUAcgBzAGkAbwBuACAAMQAuADEANQBjAG8AZABpAGMAbwBuAFQAaABlACAAaQBjAG8AbgAgAGYAbwBuAHQAIABmAG8AcgAgAFYAaQBzAHUAYQBsACAAUwB0AHUAZABpAG8AIABDAG8AZABlAGgAdAB0AHAAOgAvAC8AZgBvAG4AdABlAGwAbABvAC4AYwBvAG0AAgAAAAAAAAADAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAIEAQIBAwEEAQUBBgEHAQgBCQEKAQsBDAENAQ4BDwEQAREBEgETARQBFQEWARcBGAEZARoBGwEcAR0BHgEfASABIQEiASMBJAElASYBJwEoASkBKgErASwBLQEuAS8BMAExATIBMwE0ATUBNgE3ATgBOQE6ATsBPAE9AT4BPwFAAUEBQgFDAUQBRQFGAUcBSAFJAUoBSwFMAU0BTgFPAVABUQFSAVMBVAFVAVYBVwFYAVkBWgFbAVwBXQFeAV8BYAFhAWIBYwFkAWUBZgFnAWgBaQFqAWsBbAFtAW4BbwFwAXEBcgFzAXQBdQF2AXcBeAF5AXoBewF8AX0BfgF/AYABgQGCAYMBhAGFAYYBhwGIAYkBigGLAYwBjQGOAY8BkAGRAZIBkwGUAZUBlgGXAZgBmQGaAZsBnAGdAZ4BnwGgAaEBogGjAaQBpQGmAacBqAGpAaoBqwGsAa0BrgGvAbABsQGyAbMBtAG1AbYBtwG4AbkBugG7AbwBvQG+Ab8BwAHBAcIBwwHEAcUBxgHHAcgByQHKAcsBzAHNAc4BzwHQAdEB0gHTAdQB1QHWAdcB2AHZAdoB2wHcAd0B3gHfAeAB4QHiAeMB5AHlAeYB5wHoAekB6gHrAewB7QHuAe8B8AHxAfIB8wH0AfUB9gH3AfgB+QH6AfsB/AH9Af4B/wIAAgECAgIDAgQCBQIGAgcCCAIJAgoCCwIMAg0CDgIPAhACEQISAhMCFAIVAhYCFwIYAhkCGgIbAhwCHQIeAh8CIAIhAiICIwIkAiUCJgInAigCKQIqAisCLAItAi4CLwIwAjECMgIzAjQCNQI2AjcCOAI5AjoCOwI8Aj0CPgI/AkACQQJCAkMCRAJFAkYCRwJIAkkCSgJLAkwCTQJOAk8CUAJRAlICUwJUAlUCVgJXAlgCWQJaAlsCXAJdAl4CXwJgAmECYgJjAmQCZQJmAmcCaAJpAmoCawJsAm0CbgJvAnACcQJyAnMCdAJ1AnYCdwJ4AnkCegJ7AnwCfQJ+An8CgAKBAoICgwKEAoUChgKHAogCiQKKAosCjAKNAo4CjwKQApECkgKTApQClQKWApcCmAKZApoCmwKcAp0CngKfAqACoQKiAqMCpAKlAqYCpwKoAqkCqgKrAqwCrQKuAq8CsAKxArICswK0ArUCtgK3ArgCuQK6ArsCvAK9Ar4CvwLAAsECwgLDAsQCxQLGAscCyALJAsoCywLMAs0CzgLPAtAC0QLSAtMC1ALVAtYC1wLYAtkC2gLbAtwC3QLeAt8C4ALhAuIC4wLkAuUC5gLnAugC6QLqAusC7ALtAu4C7wLwAvEC8gLzAvQC9QL2AvcC+AL5AvoC+wL8Av0C/gL/AwADAQMCAwMDBAMFAAdhY2NvdW50FGFjdGl2YXRlLWJyZWFrcG9pbnRzA2FkZAVhZ2VudAdhcmNoaXZlCmFycm93LWJvdGgRYXJyb3ctY2lyY2xlLWRvd24RYXJyb3ctY2lyY2xlLWxlZnQSYXJyb3ctY2lyY2xlLXJpZ2h0D2Fycm93LWNpcmNsZS11cAphcnJvdy1kb3duCmFycm93LWxlZnQLYXJyb3ctcmlnaHQQYXJyb3ctc21hbGwtZG93bhBhcnJvdy1zbWFsbC1sZWZ0EWFycm93LXNtYWxsLXJpZ2h0DmFycm93LXNtYWxsLXVwCmFycm93LXN3YXAIYXJyb3ctdXAGYXR0YWNoDGF6dXJlLWRldm9wcwVhenVyZQtiZWFrZXItc3RvcAZiZWFrZXIIYmVsbC1kb3QOYmVsbC1zbGFzaC1kb3QKYmVsbC1zbGFzaARiZWxsBWJsYW5rBGJvbGQEYm9vawhib29rbWFyawticmFja2V0LWRvdA1icmFja2V0LWVycm9yCWJyaWVmY2FzZQlicm9hZGNhc3QHYnJvd3NlcgNidWcFYnVpbGQIY2FsZW5kYXINY2FsbC1pbmNvbWluZw1jYWxsLW91dGdvaW5nDmNhc2Utc2Vuc2l0aXZlEmNoYXQtc3BhcmtsZS1lcnJvchRjaGF0LXNwYXJrbGUtd2FybmluZwxjaGF0LXNwYXJrbGUJY2hlY2stYWxsBWNoZWNrCWNoZWNrbGlzdAxjaGV2cm9uLWRvd24MY2hldnJvbi1sZWZ0DWNoZXZyb24tcmlnaHQKY2hldnJvbi11cARjaGlwDGNocm9tZS1jbG9zZQ9jaHJvbWUtbWF4aW1pemUPY2hyb21lLW1pbmltaXplDmNocm9tZS1yZXN0b3JlDWNpcmNsZS1maWxsZWQTY2lyY2xlLWxhcmdlLWZpbGxlZAxjaXJjbGUtbGFyZ2UMY2lyY2xlLXNsYXNoE2NpcmNsZS1zbWFsbC1maWxsZWQMY2lyY2xlLXNtYWxsBmNpcmNsZQ1jaXJjdWl0LWJvYXJkCWNsZWFyLWFsbAZjbGlwcHkJY2xvc2UtYWxsBWNsb3NlDmNsb3VkLWRvd25sb2FkDGNsb3VkLXVwbG9hZAVjbG91ZAhjb2RlLW9zcwtjb2RlLXJldmlldwRjb2RlBmNvZmZlZQxjb2xsYXBzZS1hbGwKY29sbGVjdGlvbgpjb2xvci1tb2RlB2NvbWJpbmUYY29tbWVudC1kaXNjdXNzaW9uLXF1b3RlGmNvbW1lbnQtZGlzY3Vzc2lvbi1zcGFya2xlEmNvbW1lbnQtZGlzY3Vzc2lvbg1jb21tZW50LWRyYWZ0EmNvbW1lbnQtdW5yZXNvbHZlZAdjb21tZW50DmNvbXBhc3MtYWN0aXZlC2NvbXBhc3MtZG90B2NvbXBhc3MPY29waWxvdC1ibG9ja2VkDWNvcGlsb3QtZXJyb3ITY29waWxvdC1pbi1wcm9ncmVzcw1jb3BpbG90LWxhcmdlFWNvcGlsb3Qtbm90LWNvbm5lY3RlZA5jb3BpbG90LXNub296ZQ9jb3BpbG90LXN1Y2Nlc3MTY29waWxvdC11bmF2YWlsYWJsZRVjb3BpbG90LXdhcm5pbmctbGFyZ2UPY29waWxvdC13YXJuaW5nB2NvcGlsb3QEY29weQhjb3ZlcmFnZQtjcmVkaXQtY2FyZAZjdXJzb3IEZGFzaAlkYXNoYm9hcmQIZGF0YWJhc2UJZGVidWctYWxsD2RlYnVnLWFsdC1zbWFsbAlkZWJ1Zy1hbHQnZGVidWctYnJlYWtwb2ludC1jb25kaXRpb25hbC11bnZlcmlmaWVkHGRlYnVnLWJyZWFrcG9pbnQtY29uZGl0aW9uYWwgZGVidWctYnJlYWtwb2ludC1kYXRhLXVudmVyaWZpZWQVZGVidWctYnJlYWtwb2ludC1kYXRhJGRlYnVnLWJyZWFrcG9pbnQtZnVuY3Rpb24tdW52ZXJpZmllZBlkZWJ1Zy1icmVha3BvaW50LWZ1bmN0aW9uH2RlYnVnLWJyZWFrcG9pbnQtbG9nLXVudmVyaWZpZWQUZGVidWctYnJlYWtwb2ludC1sb2ccZGVidWctYnJlYWtwb2ludC11bnN1cHBvcnRlZA9kZWJ1Zy1jb25uZWN0ZWQNZGVidWctY29uc29sZRRkZWJ1Zy1jb250aW51ZS1zbWFsbA5kZWJ1Zy1jb3ZlcmFnZRBkZWJ1Zy1kaXNjb25uZWN0EmRlYnVnLWxpbmUtYnktbGluZQtkZWJ1Zy1wYXVzZQtkZWJ1Zy1yZXJ1bhNkZWJ1Zy1yZXN0YXJ0LWZyYW1lDWRlYnVnLXJlc3RhcnQWZGVidWctcmV2ZXJzZS1jb250aW51ZRdkZWJ1Zy1zdGFja2ZyYW1lLWFjdGl2ZRBkZWJ1Zy1zdGFja2ZyYW1lC2RlYnVnLXN0YXJ0D2RlYnVnLXN0ZXAtYmFjaw9kZWJ1Zy1zdGVwLWludG8OZGVidWctc3RlcC1vdXQPZGVidWctc3RlcC1vdmVyCmRlYnVnLXN0b3AFZGVidWcQZGVza3RvcC1kb3dubG9hZBNkZXZpY2UtY2FtZXJhLXZpZGVvDWRldmljZS1jYW1lcmENZGV2aWNlLW1vYmlsZQpkaWZmLWFkZGVkDGRpZmYtaWdub3JlZA1kaWZmLW1vZGlmaWVkDWRpZmYtbXVsdGlwbGUMZGlmZi1yZW1vdmVkDGRpZmYtcmVuYW1lZAtkaWZmLXNpbmdsZQRkaWZmB2Rpc2NhcmQJZWRpdC1jb2RlDGVkaXQtc2Vzc2lvbgxlZGl0LXNwYXJrbGUEZWRpdA1lZGl0b3ItbGF5b3V0CGVsbGlwc2lzDGVtcHR5LXdpbmRvdwZlcmFzZXILZXJyb3Itc21hbGwFZXJyb3IHZXhjbHVkZQpleHBhbmQtYWxsBmV4cG9ydBBleHRlbnNpb25zLWxhcmdlCmV4dGVuc2lvbnMKZXllLWNsb3NlZANleWUIZmVlZGJhY2sLZmlsZS1iaW5hcnkJZmlsZS1jb2RlCmZpbGUtbWVkaWEIZmlsZS1wZGYOZmlsZS1zdWJtb2R1bGUWZmlsZS1zeW1saW5rLWRpcmVjdG9yeRFmaWxlLXN5bWxpbmstZmlsZQlmaWxlLXRleHQIZmlsZS16aXAEZmlsZQVmaWxlcw1maWx0ZXItZmlsbGVkBmZpbHRlcgRmbGFnBWZsYW1lCWZvbGQtZG93bgdmb2xkLXVwBGZvbGQNZm9sZGVyLWFjdGl2ZQ5mb2xkZXItbGlicmFyeQ1mb2xkZXItb3BlbmVkBmZvbGRlcgRnYW1lBGdlYXIEZ2lmdAtnaXN0LXNlY3JldARnaXN0EmdpdC1icmFuY2gtY2hhbmdlcxRnaXQtYnJhbmNoLWNvbmZsaWN0cxlnaXQtYnJhbmNoLXN0YWdlZC1jaGFuZ2VzCmdpdC1icmFuY2gKZ2l0LWNvbW1pdAtnaXQtY29tcGFyZQlnaXQtZmV0Y2gIZ2l0LWxlbnMJZ2l0LW1lcmdlF2dpdC1wdWxsLXJlcXVlc3QtY2xvc2VkF2dpdC1wdWxsLXJlcXVlc3QtY3JlYXRlFWdpdC1wdWxsLXJlcXVlc3QtZG9uZRZnaXQtcHVsbC1yZXF1ZXN0LWRyYWZ0HmdpdC1wdWxsLXJlcXVlc3QtZ28tdG8tY2hhbmdlcxxnaXQtcHVsbC1yZXF1ZXN0LW5ldy1jaGFuZ2VzEGdpdC1wdWxsLXJlcXVlc3QPZ2l0LXN0YXNoLWFwcGx5DWdpdC1zdGFzaC1wb3AJZ2l0LXN0YXNoDWdpdGh1Yi1hY3Rpb24KZ2l0aHViLWFsdA9naXRodWItaW52ZXJ0ZWQOZ2l0aHViLXByb2plY3QGZ2l0aHViBWdsb2JlFWdvLXRvLWVkaXRpbmctc2Vzc2lvbgpnby10by1maWxlDGdvLXRvLXNlYXJjaAdncmFiYmVyCmdyYXBoLWxlZnQKZ3JhcGgtbGluZQ1ncmFwaC1zY2F0dGVyBWdyYXBoB2dyaXBwZXIRZ3JvdXAtYnktcmVmLXR5cGUMaGVhcnQtZmlsbGVkBWhlYXJ0B2hpc3RvcnkEaG9tZQ9ob3Jpem9udGFsLXJ1bGUFaHVib3QFaW5ib3gGaW5kZW50CmluZGV4LXplcm8EaW5mbwZpbnNlcnQHaW5zcGVjdAtpc3N1ZS1kcmFmdA5pc3N1ZS1yZW9wZW5lZAZpc3N1ZXMGaXRhbGljBmplcnNleQRqc29uDmtlYmFiLXZlcnRpY2FsA2tleRJrZXlib2FyZC10YWItYWJvdmUSa2V5Ym9hcmQtdGFiLWJlbG93DGtleWJvYXJkLXRhYgNsYXcNbGF5ZXJzLWFjdGl2ZQpsYXllcnMtZG90BmxheWVycxdsYXlvdXQtYWN0aXZpdHliYXItbGVmdBhsYXlvdXQtYWN0aXZpdHliYXItcmlnaHQPbGF5b3V0LWNlbnRlcmVkDmxheW91dC1tZW51YmFyE2xheW91dC1wYW5lbC1jZW50ZXIRbGF5b3V0LXBhbmVsLWRvY2sUbGF5b3V0LXBhbmVsLWp1c3RpZnkRbGF5b3V0LXBhbmVsLWxlZnQQbGF5b3V0LXBhbmVsLW9mZhJsYXlvdXQtcGFuZWwtcmlnaHQMbGF5b3V0LXBhbmVsGGxheW91dC1zaWRlYmFyLWxlZnQtZG9jaxdsYXlvdXQtc2lkZWJhci1sZWZ0LW9mZhNsYXlvdXQtc2lkZWJhci1sZWZ0GWxheW91dC1zaWRlYmFyLXJpZ2h0LWRvY2sYbGF5b3V0LXNpZGViYXItcmlnaHQtb2ZmFGxheW91dC1zaWRlYmFyLXJpZ2h0EGxheW91dC1zdGF0dXNiYXIGbGF5b3V0B2xpYnJhcnkRbGlnaHRidWxiLWF1dG9maXgPbGlnaHRidWxiLWVtcHR5EWxpZ2h0YnVsYi1zcGFya2xlCWxpZ2h0YnVsYg1saW5rLWV4dGVybmFsBGxpbmsJbGlzdC1mbGF0DGxpc3Qtb3JkZXJlZA5saXN0LXNlbGVjdGlvbglsaXN0LXRyZWUObGlzdC11bm9yZGVyZWQKbGl2ZS1zaGFyZQdsb2FkaW5nCGxvY2F0aW9uCmxvY2stc21hbGwEbG9jawZtYWduZXQJbWFpbC1yZWFkBG1haWwKbWFwLWZpbGxlZBNtYXAtdmVydGljYWwtZmlsbGVkDG1hcC12ZXJ0aWNhbANtYXAIbWFya2Rvd24DbWNwCW1lZ2FwaG9uZQdtZW50aW9uBG1lbnUKbWVyZ2UtaW50bwVtZXJnZQptaWMtZmlsbGVkA21pYwltaWxlc3RvbmUGbWlycm9yDG1vcnRhci1ib2FyZARtb3ZlEG11bHRpcGxlLXdpbmRvd3MFbXVzaWMEbXV0ZQ5uZXctY29sbGVjdGlvbghuZXctZmlsZQpuZXctZm9sZGVyB25ld2xpbmUKbm8tbmV3bGluZQRub3RlEW5vdGVib29rLXRlbXBsYXRlCG5vdGVib29rCG9jdG9mYWNlD29wZW4taW4tcHJvZHVjdAxvcGVuLXByZXZpZXcMb3JnYW5pemF0aW9uBm91dHB1dAdwYWNrYWdlCHBhaW50Y2FuC3Bhc3MtZmlsbGVkBHBhc3MKcGVyY2VudGFnZQpwZXJzb24tYWRkBnBlcnNvbgVwaWFubwlwaWUtY2hhcnQDcGluDHBpbm5lZC1kaXJ0eQZwaW5uZWQLcGxheS1jaXJjbGUEcGx1Zw1wcmVzZXJ2ZS1jYXNlB3ByZXZpZXcQcHJpbWl0aXZlLXNxdWFyZQdwcm9qZWN0BXB1bHNlBnB5dGhvbghxdWVzdGlvbgVxdW90ZQZxdW90ZXMLcmFkaW8tdG93ZXIJcmVhY3Rpb25zC3JlY29yZC1rZXlzDHJlY29yZC1zbWFsbAZyZWNvcmQEcmVkbwpyZWZlcmVuY2VzB3JlZnJlc2gFcmVnZXgPcmVtb3RlLWV4cGxvcmVyBnJlbW90ZQZyZW1vdmUGcmVuYW1lC3JlcGxhY2UtYWxsB3JlcGxhY2UFcmVwbHkKcmVwby1jbG9uZQpyZXBvLWZldGNoD3JlcG8tZm9yY2UtcHVzaAtyZXBvLWZvcmtlZAtyZXBvLXBpbm5lZAlyZXBvLXB1bGwJcmVwby1wdXNoDXJlcG8tc2VsZWN0ZWQEcmVwbwZyZXBvcnQFcm9ib3QGcm9ja2V0EnJvb3QtZm9sZGVyLW9wZW5lZAtyb290LWZvbGRlcgNyc3MEcnVieQlydW4tYWJvdmUQcnVuLWFsbC1jb3ZlcmFnZQdydW4tYWxsCXJ1bi1iZWxvdwxydW4tY292ZXJhZ2UKcnVuLWVycm9ycw1ydW4td2l0aC1kZXBzCHNhdmUtYWxsB3NhdmUtYXMEc2F2ZQtzY3JlZW4tZnVsbA1zY3JlZW4tbm9ybWFsDHNlYXJjaC1mdXp6eQxzZWFyY2gtbGFyZ2UOc2VhcmNoLXNwYXJrbGULc2VhcmNoLXN0b3AGc2VhcmNoFHNlbmQtdG8tcmVtb3RlLWFnZW50BHNlbmQSc2VydmVyLWVudmlyb25tZW50DnNlcnZlci1wcm9jZXNzBnNlcnZlcg1zZXR0aW5ncy1nZWFyCHNldHRpbmdzBXNoYXJlBnNoaWVsZAdzaWduLWluCHNpZ24tb3V0BHNraXAGc21pbGV5BXNuYWtlD3NvcnQtcHJlY2VkZW5jZQ5zcGFya2xlLWZpbGxlZAdzcGFya2xlEHNwbGl0LWhvcml6b250YWwOc3BsaXQtdmVydGljYWwIc3F1aXJyZWwKc3Rhci1lbXB0eQlzdGFyLWZ1bGwJc3Rhci1oYWxmC3N0b3AtY2lyY2xlDXN0cmlrZXRocm91Z2gNc3Vycm91bmQtd2l0aAxzeW1ib2wtYXJyYXkOc3ltYm9sLWJvb2xlYW4Mc3ltYm9sLWNsYXNzDHN5bWJvbC1jb2xvcg9zeW1ib2wtY29uc3RhbnQSc3ltYm9sLWVudW0tbWVtYmVyC3N5bWJvbC1lbnVtDHN5bWJvbC1ldmVudAxzeW1ib2wtZmllbGQQc3ltYm9sLWludGVyZmFjZQpzeW1ib2wta2V5DnN5bWJvbC1rZXl3b3JkE3N5bWJvbC1tZXRob2QtYXJyb3cNc3ltYm9sLW1ldGhvZAtzeW1ib2wtbWlzYw5zeW1ib2wtbnVtZXJpYw9zeW1ib2wtb3BlcmF0b3IQc3ltYm9sLXBhcmFtZXRlcg9zeW1ib2wtcHJvcGVydHkMc3ltYm9sLXJ1bGVyDnN5bWJvbC1zbmlwcGV0EHN5bWJvbC1zdHJ1Y3R1cmUPc3ltYm9sLXZhcmlhYmxlDHN5bmMtaWdub3JlZARzeW5jBXRhYmxlA3RhZwZ0YXJnZXQIdGFza2xpc3QJdGVsZXNjb3BlDXRlcm1pbmFsLWJhc2gMdGVybWluYWwtY21kD3Rlcm1pbmFsLWRlYmlhbhF0ZXJtaW5hbC1naXQtYmFzaA50ZXJtaW5hbC1saW51eBN0ZXJtaW5hbC1wb3dlcnNoZWxsDXRlcm1pbmFsLXRtdXgPdGVybWluYWwtdWJ1bnR1CHRlcm1pbmFsCXRleHQtc2l6ZQh0aGlua2luZwp0aHJlZS1iYXJzEXRodW1ic2Rvd24tZmlsbGVkCnRodW1ic2Rvd24PdGh1bWJzdXAtZmlsbGVkCHRodW1ic3VwBXRvb2xzBXRyYXNoDXRyaWFuZ2xlLWRvd24NdHJpYW5nbGUtbGVmdA50cmlhbmdsZS1yaWdodAt0cmlhbmdsZS11cAd0d2l0dGVyEnR5cGUtaGllcmFyY2h5LXN1YhR0eXBlLWhpZXJhcmNoeS1zdXBlcg50eXBlLWhpZXJhcmNoeQZ1bmZvbGQTdW5ncm91cC1ieS1yZWYtdHlwZQZ1bmxvY2sGdW5tdXRlCnVudmVyaWZpZWQOdmFyaWFibGUtZ3JvdXAPdmVyaWZpZWQtZmlsbGVkCHZlcmlmaWVkCXZtLWFjdGl2ZQp2bS1jb25uZWN0CnZtLW91dGxpbmUKdm0tcGVuZGluZwp2bS1ydW5uaW5nAnZtAnZyD3ZzY29kZS1pbnNpZGVycwZ2c2NvZGUEd2FuZAd3YXJuaW5nBXdhdGNoCndoaXRlc3BhY2UKd2hvbGUtd29yZA13aW5kb3ctYWN0aXZlCXdvcmQtd3JhcBF3b3Jrc3BhY2UtdHJ1c3RlZBF3b3Jrc3BhY2UtdW5rbm93bhN3b3Jrc3BhY2UtdW50cnVzdGVkB3pvb20taW4Iem9vbS1vdXQAAA==) format(\"truetype\")}.codicon[class*=codicon-]{font: 16px/1 codicon;display:inline-block;text-decoration:none;text-rendering:auto;text-align:center;text-transform:none;-webkit-font-smoothing:antialiased;-moz-osx-font-smoothing:grayscale;user-select:none;-webkit-user-select:none}.codicon-wrench-subaction{opacity:.5}@keyframes codicon-spin{to{transform:rotate(360deg)}}.codicon-sync.codicon-modifier-spin,.codicon-loading.codicon-modifier-spin,.codicon-gear.codicon-modifier-spin,.codicon-notebook-state-executing.codicon-modifier-spin{animation:codicon-spin 1.5s steps(30) infinite}.codicon-modifier-disabled{opacity:.4}.codicon-loading,.codicon-tree-item-loading:before{animation-duration:1s!important;animation-timing-function:cubic-bezier(.53,.21,.29,.67)!important}.monaco-editor .codicon.codicon-symbol-array,.monaco-workbench .codicon.codicon-symbol-array{color:var(--vscode-symbolIcon-arrayForeground)}.monaco-editor .codicon.codicon-symbol-boolean,.monaco-workbench .codicon.codicon-symbol-boolean{color:var(--vscode-symbolIcon-booleanForeground)}.monaco-editor .codicon.codicon-symbol-class,.monaco-workbench .codicon.codicon-symbol-class{color:var(--vscode-symbolIcon-classForeground)}.monaco-editor .codicon.codicon-symbol-method,.monaco-workbench .codicon.codicon-symbol-method{color:var(--vscode-symbolIcon-methodForeground)}.monaco-editor .codicon.codicon-symbol-color,.monaco-workbench .codicon.codicon-symbol-color{color:var(--vscode-symbolIcon-colorForeground)}.monaco-editor .codicon.codicon-symbol-constant,.monaco-workbench .codicon.codicon-symbol-constant{color:var(--vscode-symbolIcon-constantForeground)}.monaco-editor .codicon.codicon-symbol-constructor,.monaco-workbench .codicon.codicon-symbol-constructor{color:var(--vscode-symbolIcon-constructorForeground)}.monaco-editor .codicon.codicon-symbol-value,.monaco-workbench .codicon.codicon-symbol-value,.monaco-editor .codicon.codicon-symbol-enum,.monaco-workbench .codicon.codicon-symbol-enum{color:var(--vscode-symbolIcon-enumeratorForeground)}.monaco-editor .codicon.codicon-symbol-enum-member,.monaco-workbench .codicon.codicon-symbol-enum-member{color:var(--vscode-symbolIcon-enumeratorMemberForeground)}.monaco-editor .codicon.codicon-symbol-event,.monaco-workbench .codicon.codicon-symbol-event{color:var(--vscode-symbolIcon-eventForeground)}.monaco-editor .codicon.codicon-symbol-field,.monaco-workbench .codicon.codicon-symbol-field{color:var(--vscode-symbolIcon-fieldForeground)}.monaco-editor .codicon.codicon-symbol-file,.monaco-workbench .codicon.codicon-symbol-file{color:var(--vscode-symbolIcon-fileForeground)}.monaco-editor .codicon.codicon-symbol-folder,.monaco-workbench .codicon.codicon-symbol-folder{color:var(--vscode-symbolIcon-folderForeground)}.monaco-editor .codicon.codicon-symbol-function,.monaco-workbench .codicon.codicon-symbol-function{color:var(--vscode-symbolIcon-functionForeground)}.monaco-editor .codicon.codicon-symbol-interface,.monaco-workbench .codicon.codicon-symbol-interface{color:var(--vscode-symbolIcon-interfaceForeground)}.monaco-editor .codicon.codicon-symbol-key,.monaco-workbench .codicon.codicon-symbol-key{color:var(--vscode-symbolIcon-keyForeground)}.monaco-editor .codicon.codicon-symbol-keyword,.monaco-workbench .codicon.codicon-symbol-keyword{color:var(--vscode-symbolIcon-keywordForeground)}.monaco-editor .codicon.codicon-symbol-module,.monaco-workbench .codicon.codicon-symbol-module{color:var(--vscode-symbolIcon-moduleForeground)}.monaco-editor .codicon.codicon-symbol-namespace,.monaco-workbench .codicon.codicon-symbol-namespace{color:var(--vscode-symbolIcon-namespaceForeground)}.monaco-editor .codicon.codicon-symbol-null,.monaco-workbench .codicon.codicon-symbol-null{color:var(--vscode-symbolIcon-nullForeground)}.monaco-editor .codicon.codicon-symbol-number,.monaco-workbench .codicon.codicon-symbol-number{color:var(--vscode-symbolIcon-numberForeground)}.monaco-editor .codicon.codicon-symbol-object,.monaco-workbench .codicon.codicon-symbol-object{color:var(--vscode-symbolIcon-objectForeground)}.monaco-editor .codicon.codicon-symbol-operator,.monaco-workbench .codicon.codicon-symbol-operator{color:var(--vscode-symbolIcon-operatorForeground)}.monaco-editor .codicon.codicon-symbol-package,.monaco-workbench .codicon.codicon-symbol-package{color:var(--vscode-symbolIcon-packageForeground)}.monaco-editor .codicon.codicon-symbol-property,.monaco-workbench .codicon.codicon-symbol-property{color:var(--vscode-symbolIcon-propertyForeground)}.monaco-editor .codicon.codicon-symbol-reference,.monaco-workbench .codicon.codicon-symbol-reference{color:var(--vscode-symbolIcon-referenceForeground)}.monaco-editor .codicon.codicon-symbol-snippet,.monaco-workbench .codicon.codicon-symbol-snippet{color:var(--vscode-symbolIcon-snippetForeground)}.monaco-editor .codicon.codicon-symbol-string,.monaco-workbench .codicon.codicon-symbol-string{color:var(--vscode-symbolIcon-stringForeground)}.monaco-editor .codicon.codicon-symbol-struct,.monaco-workbench .codicon.codicon-symbol-struct{color:var(--vscode-symbolIcon-structForeground)}.monaco-editor .codicon.codicon-symbol-text,.monaco-workbench .codicon.codicon-symbol-text{color:var(--vscode-symbolIcon-textForeground)}.monaco-editor .codicon.codicon-symbol-type-parameter,.monaco-workbench .codicon.codicon-symbol-type-parameter{color:var(--vscode-symbolIcon-typeParameterForeground)}.monaco-editor .codicon.codicon-symbol-unit,.monaco-workbench .codicon.codicon-symbol-unit{color:var(--vscode-symbolIcon-unitForeground)}.monaco-editor .codicon.codicon-symbol-variable,.monaco-workbench .codicon.codicon-symbol-variable{color:var(--vscode-symbolIcon-variableForeground)}.monaco-editor .lightBulbWidget{display:flex;align-items:center;justify-content:center}.monaco-editor .lightBulbWidget:hover{cursor:pointer}.monaco-editor .lightBulbWidget.codicon-light-bulb,.monaco-editor .lightBulbWidget.codicon-lightbulb-sparkle{color:var(--vscode-editorLightBulb-foreground)}.monaco-editor .lightBulbWidget.codicon-lightbulb-autofix,.monaco-editor .lightBulbWidget.codicon-lightbulb-sparkle-autofix{color:var(--vscode-editorLightBulbAutoFix-foreground, var(--vscode-editorLightBulb-foreground))}.monaco-editor .lightBulbWidget.codicon-sparkle-filled{color:var(--vscode-editorLightBulbAi-foreground, var(--vscode-icon-foreground))}.monaco-editor .lightBulbWidget:before{position:relative;z-index:2}.monaco-editor .lightBulbWidget:after{position:absolute;top:0;left:0;content:\"\";display:block;width:100%;height:100%;opacity:.3;z-index:1}.monaco-editor .glyph-margin-widgets .cgmr[class*=codicon-gutter-lightbulb]{display:block;cursor:pointer}.monaco-editor .glyph-margin-widgets .cgmr.codicon-gutter-lightbulb,.monaco-editor .glyph-margin-widgets .cgmr.codicon-gutter-lightbulb-sparkle{color:var(--vscode-editorLightBulb-foreground)}.monaco-editor .glyph-margin-widgets .cgmr.codicon-gutter-lightbulb-auto-fix,.monaco-editor .glyph-margin-widgets .cgmr.codicon-gutter-lightbulb-aifix-auto-fix{color:var(--vscode-editorLightBulbAutoFix-foreground, var(--vscode-editorLightBulb-foreground))}.monaco-editor .glyph-margin-widgets .cgmr.codicon-gutter-lightbulb-sparkle-filled{color:var(--vscode-editorLightBulbAi-foreground, var(--vscode-icon-foreground))}.monaco-editor .codelens-decoration{overflow:hidden;display:inline-flex!important;align-items:center;text-overflow:ellipsis;white-space:nowrap;color:var(--vscode-editorCodeLens-foreground);line-height:var(--vscode-editorCodeLens-lineHeight);font-size:var(--vscode-editorCodeLens-fontSize);padding-right:calc(var(--vscode-editorCodeLens-fontSize)*.5);font-feature-settings:var(--vscode-editorCodeLens-fontFeatureSettings);font-family:var(--vscode-editorCodeLens-fontFamily),var(--vscode-editorCodeLens-fontFamilyDefault)}.monaco-editor .codelens-decoration>span,.monaco-editor .codelens-decoration>a{user-select:none;-webkit-user-select:none;white-space:nowrap;vertical-align:sub;display:inline-flex;align-items:center}.monaco-editor .codelens-decoration>a{text-decoration:none}.monaco-editor .codelens-decoration>a:hover{cursor:pointer;color:var(--vscode-editorLink-activeForeground)!important}.monaco-editor .codelens-decoration>a:hover .codicon{color:var(--vscode-editorLink-activeForeground)!important}.monaco-editor .codelens-decoration .codicon[class*=codicon-]{vertical-align:middle;color:currentColor!important;color:var(--vscode-editorCodeLens-foreground);line-height:var(--vscode-editorCodeLens-lineHeight);font-size:var(--vscode-editorCodeLens-fontSize)}.monaco-editor .codelens-decoration>a:hover .codicon:before{cursor:pointer}@keyframes fadein{0%{opacity:0}to{opacity:1}}.monaco-editor .codelens-decoration.fadein{animation:fadein .1s linear}.monaco-editor .inlineSuggestionsHints{padding:4px;.warningMessage p{margin:0}}.monaco-editor .inlineSuggestionsHints.withBorder{z-index:39;color:var(--vscode-editorHoverWidget-foreground);background-color:var(--vscode-editorHoverWidget-background);border:1px solid var(--vscode-editorHoverWidget-border)}.monaco-editor .inlineSuggestionsHints a,.monaco-editor .inlineSuggestionsHints a:hover{color:var(--vscode-foreground)!important}.monaco-editor .inlineSuggestionsHints .keybinding{display:flex;margin-left:4px;opacity:.6}.monaco-editor .inlineSuggestionsHints .keybinding .monaco-keybinding-key{font-size:8px;padding:2px 3px}.monaco-editor .inlineSuggestionsHints .availableSuggestionCount a{display:flex;min-width:19px;justify-content:center}.monaco-editor .inlineSuggestionStatusBarItemLabel{margin-right:2px}.monaco-hover{cursor:default;position:absolute;overflow:hidden;user-select:text;-webkit-user-select:text;box-sizing:border-box;line-height:1.5em;white-space:var(--vscode-hover-whiteSpace, normal)}.monaco-hover.fade-in{animation:fadein .1s linear}.monaco-hover.hidden{display:none}.monaco-hover a:hover:not(.disabled){cursor:pointer}.monaco-hover .hover-contents:not(.html-hover-contents){padding:4px 8px}.monaco-hover .markdown-hover>.hover-contents:not(.code-hover-contents){max-width:var(--vscode-hover-maxWidth, 500px);word-wrap:break-word}.monaco-hover .markdown-hover>.hover-contents:not(.code-hover-contents) hr{min-width:100%}.monaco-hover p,.monaco-hover .code,.monaco-hover ul,.monaco-hover h1,.monaco-hover h2,.monaco-hover h3,.monaco-hover h4,.monaco-hover h5,.monaco-hover h6{margin:8px 0}.monaco-hover h1,.monaco-hover h2,.monaco-hover h3,.monaco-hover h4,.monaco-hover h5,.monaco-hover h6{line-height:1.1}.monaco-hover code{font-family:var(--monaco-monospace-font)}.monaco-hover hr{box-sizing:border-box;border-left:0px;border-right:0px;margin:4px -8px -4px;height:1px}.monaco-hover p:first-child,.monaco-hover .code:first-child,.monaco-hover ul:first-child{margin-top:0}.monaco-hover p:last-child,.monaco-hover .code:last-child,.monaco-hover ul:last-child{margin-bottom:0}.monaco-hover ul,.monaco-hover ol{padding-left:20px}.monaco-hover li>p{margin-bottom:0}.monaco-hover li>ul{margin-top:0}.monaco-hover code{border-radius:3px;padding:0 .4em}.monaco-hover .monaco-tokenized-source{white-space:var(--vscode-hover-sourceWhiteSpace, pre-wrap)}.monaco-hover .hover-row.status-bar{font-size:12px;line-height:22px}.monaco-hover .hover-row.status-bar .info{font-style:italic;padding:0 8px}.monaco-hover .hover-row.status-bar .actions{display:flex;padding:0 8px;width:100%}.monaco-hover .hover-row.status-bar .actions .action-container{margin-right:16px;cursor:pointer;overflow:hidden;text-wrap:nowrap;text-overflow:ellipsis}.monaco-hover .hover-row.status-bar .actions .action-container .action .icon{padding-right:4px;vertical-align:middle}.monaco-hover .hover-row.status-bar .actions .action-container a{color:var(--vscode-textLink-foreground);text-decoration:var(--text-link-decoration)}.monaco-hover .hover-row.status-bar .actions .action-container a .icon.codicon{color:var(--vscode-textLink-foreground)}.monaco-hover .markdown-hover .hover-contents .codicon{color:inherit;font-size:inherit;vertical-align:middle}.monaco-hover .hover-contents a.code-link:hover,.monaco-hover .hover-contents a.code-link{color:inherit}.monaco-hover .hover-contents a.code-link:before{content:\"(\"}.monaco-hover .hover-contents a.code-link:after{content:\")\"}.monaco-hover .hover-contents a.code-link>span{text-decoration:underline;border-bottom:1px solid transparent;text-underline-position:under;color:var(--vscode-textLink-foreground)}.monaco-hover .hover-contents a.code-link>span:hover{color:var(--vscode-textLink-activeForeground)}.monaco-hover .markdown-hover .hover-contents:not(.code-hover-contents):not(.html-hover-contents) p:last-child [style*=background-color]{margin-bottom:4px;display:inline-block}.monaco-hover .markdown-hover .hover-contents:not(.code-hover-contents):not(.html-hover-contents) span.codicon{margin-bottom:2px}.monaco-hover-content .action-container a{-webkit-user-select:none;user-select:none}.monaco-hover-content .action-container.disabled{pointer-events:none;opacity:.4;cursor:default}.monaco-hover .action-container,.monaco-hover .action,.monaco-hover button,.monaco-hover .monaco-button,.monaco-hover .monaco-text-button,.monaco-hover [role=button]{-webkit-user-select:none;user-select:none}.monaco-custom-toggle{margin-left:2px;float:left;cursor:pointer;overflow:hidden;width:20px;height:20px;border-radius:3px;border:1px solid transparent;padding:1px;box-sizing:border-box;user-select:none;-webkit-user-select:none}.monaco-custom-toggle:hover{background-color:var(--vscode-inputOption-hoverBackground)}.hc-black .monaco-custom-toggle:hover,.hc-light .monaco-custom-toggle:hover{border:1px dashed var(--vscode-focusBorder)}.hc-black .monaco-custom-toggle,.hc-light .monaco-custom-toggle,.hc-black .monaco-custom-toggle:hover,.hc-light .monaco-custom-toggle:hover{background:none}.monaco-custom-toggle.monaco-checkbox{height:18px;width:18px;border:1px solid transparent;border-radius:3px;margin-right:9px;margin-left:0;padding:0;opacity:1;background-size:16px!important}.monaco-action-bar .checkbox-action-item{display:flex;align-items:center;border-radius:2px;padding-right:2px}.monaco-action-bar .checkbox-action-item:hover{background-color:var(--vscode-toolbar-hoverBackground)}.monaco-action-bar .checkbox-action-item>.monaco-custom-toggle.monaco-checkbox{margin-right:4px}.monaco-action-bar .checkbox-action-item>.checkbox-label{font-size:12px}.monaco-editor .find-widget{position:absolute;z-index:35;height:33px;overflow:hidden;line-height:19px;transition:transform .2s linear;padding:0 4px;box-sizing:border-box;transform:translateY(calc(-100% - 10px));box-shadow:0 0 8px 2px var(--vscode-widget-shadow);color:var(--vscode-editorWidget-foreground);border-left:1px solid var(--vscode-widget-border);border-right:1px solid var(--vscode-widget-border);border-bottom:1px solid var(--vscode-widget-border);border-bottom-left-radius:4px;border-bottom-right-radius:4px;background-color:var(--vscode-editorWidget-background)}.monaco-reduce-motion .monaco-editor .find-widget{transition:transform 0ms linear}.monaco-editor .find-widget textarea{margin:0}.monaco-editor .find-widget.hiddenEditor{display:none}.monaco-editor .find-widget.replaceToggled>.replace-part{display:flex}.monaco-editor .find-widget.visible{transform:translateY(0)}.monaco-editor .find-widget .monaco-inputbox.synthetic-focus{outline:1px solid -webkit-focus-ring-color;outline-offset:-1px;outline-color:var(--vscode-focusBorder)}.monaco-editor .find-widget .monaco-inputbox .input{background-color:transparent;min-height:0}.monaco-editor .find-widget .monaco-findInput .input{font-size:13px}.monaco-editor .find-widget>.find-part,.monaco-editor .find-widget>.replace-part{margin:3px 25px 0 17px;font-size:12px;display:flex}.monaco-editor .find-widget>.find-part .monaco-inputbox,.monaco-editor .find-widget>.replace-part .monaco-inputbox{min-height:25px}.monaco-editor .find-widget>.replace-part .monaco-inputbox>.ibwrapper>.mirror{padding-right:22px}.monaco-editor .find-widget>.find-part .monaco-inputbox>.ibwrapper>.input,.monaco-editor .find-widget>.find-part .monaco-inputbox>.ibwrapper>.mirror,.monaco-editor .find-widget>.replace-part .monaco-inputbox>.ibwrapper>.input,.monaco-editor .find-widget>.replace-part .monaco-inputbox>.ibwrapper>.mirror{padding-top:2px;padding-bottom:2px}.monaco-editor .find-widget>.find-part .find-actions{height:25px;display:flex;align-items:center}.monaco-editor .find-widget>.replace-part .replace-actions{height:25px;display:flex;align-items:center}.monaco-editor .find-widget .monaco-findInput{vertical-align:middle;display:flex;flex:1}.monaco-editor .find-widget .monaco-findInput .monaco-scrollable-element{width:100%}.monaco-editor .find-widget .monaco-findInput .monaco-scrollable-element .scrollbar.vertical{opacity:0}.monaco-editor .find-widget .matchesCount{display:flex;flex:initial;margin:0 0 0 3px;padding:2px 0 0 2px;height:25px;vertical-align:middle;box-sizing:border-box;text-align:center;line-height:23px}.monaco-editor .find-widget .button{width:16px;height:16px;padding:3px;border-radius:5px;flex:initial;margin-left:3px;background-position:center center;background-repeat:no-repeat;cursor:pointer;display:flex;align-items:center;justify-content:center}.monaco-editor .find-widget .codicon-find-selection{width:22px;height:22px;padding:3px;border-radius:5px}.monaco-editor .find-widget .button.left{margin-left:0;margin-right:3px}.monaco-editor .find-widget .button.wide{width:auto;padding:1px 6px;top:-1px}.monaco-editor .find-widget .button.toggle{position:absolute;top:0;left:3px;width:18px;height:100%;border-radius:0;box-sizing:border-box}.monaco-editor .find-widget .button.toggle.disabled{display:none}.monaco-editor .find-widget .disabled{color:var(--vscode-disabledForeground);cursor:default}.monaco-editor .find-widget>.replace-part{display:none}.monaco-editor .find-widget>.replace-part>.monaco-findInput{position:relative;display:flex;vertical-align:middle;flex:auto;flex-grow:0;flex-shrink:0}.monaco-editor .find-widget>.replace-part>.monaco-findInput>.controls{position:absolute;top:3px;right:2px}.monaco-editor .find-widget.reduced-find-widget .matchesCount{display:none}.monaco-editor .find-widget.narrow-find-widget{max-width:257px!important}.monaco-editor .find-widget.collapsed-find-widget{max-width:170px!important}.monaco-editor .find-widget.collapsed-find-widget .button.previous,.monaco-editor .find-widget.collapsed-find-widget .button.next,.monaco-editor .find-widget.collapsed-find-widget .button.replace,.monaco-editor .find-widget.collapsed-find-widget .button.replace-all,.monaco-editor .find-widget.collapsed-find-widget>.find-part .monaco-findInput .controls{display:none}.monaco-editor .find-widget.no-results .matchesCount{color:var(--vscode-errorForeground)}.monaco-editor .findMatch{animation-duration:0;animation-name:inherit!important;background-color:var(--vscode-editor-findMatchHighlightBackground)}.monaco-editor .currentFindMatch{background-color:var(--vscode-editor-findMatchBackground);border:2px solid var(--vscode-editor-findMatchBorder);padding:1px;box-sizing:border-box}.monaco-editor .findScope{background-color:var(--vscode-editor-findRangeHighlightBackground)}.monaco-editor .find-widget .monaco-sash{left:0!important;background-color:var(--vscode-editorWidget-resizeBorder, var(--vscode-editorWidget-border))}.monaco-editor.hc-black .find-widget .button:before{position:relative;top:1px;left:2px}.monaco-editor .find-widget .button:not(.disabled):hover,.monaco-editor .find-widget .codicon-find-selection:hover{background-color:var(--vscode-toolbar-hoverBackground)!important}.monaco-editor.findMatch{background-color:var(--vscode-editor-findMatchHighlightBackground)}.monaco-editor.currentFindMatch{background-color:var(--vscode-editor-findMatchBackground)}.monaco-editor.findScope{background-color:var(--vscode-editor-findRangeHighlightBackground)}.monaco-editor.findMatch{background-color:var(--vscode-editorWidget-background)}.monaco-editor .find-widget>.button.codicon-widget-close{position:absolute;top:5px;right:4px}.monaco-inputbox{position:relative;display:block;padding:0;box-sizing:border-box;border-radius:2px;font-size:inherit}.monaco-inputbox>.ibwrapper>.input,.monaco-inputbox>.ibwrapper>.mirror{padding:4px 6px}.monaco-inputbox>.ibwrapper{position:relative;width:100%}.monaco-inputbox>.ibwrapper>.input{display:inline-block;box-sizing:border-box;width:100%;height:100%;line-height:inherit;border:none;font-family:inherit;font-size:inherit;resize:none;color:inherit}.monaco-inputbox>.ibwrapper>input{text-overflow:ellipsis}.monaco-inputbox>.ibwrapper>textarea.input{display:block;scrollbar-width:none;outline:none}.monaco-inputbox>.ibwrapper>textarea.input::-webkit-scrollbar{display:none}.monaco-inputbox>.ibwrapper>textarea.input.empty{white-space:nowrap}.monaco-inputbox>.ibwrapper>.mirror{position:absolute;display:inline-block;width:100%;top:0;left:0;box-sizing:border-box;white-space:pre-wrap;visibility:hidden;word-wrap:break-word}.monaco-inputbox-container{text-align:right}.monaco-inputbox-container .monaco-inputbox-message{display:inline-block;overflow:hidden;text-align:left;width:100%;box-sizing:border-box;padding:.4em;font-size:12px;line-height:17px;margin-top:-1px;word-wrap:break-word}.monaco-inputbox .monaco-action-bar{position:absolute;right:2px;top:4px}.monaco-inputbox .monaco-action-bar .action-item{margin-left:2px}.monaco-inputbox .monaco-action-bar .action-item .codicon{background-repeat:no-repeat;width:16px;height:16px}.monaco-findInput{position:relative}.monaco-findInput .monaco-inputbox{font-size:13px;width:100%}.monaco-findInput>.controls{position:absolute;top:3px;right:2px}.vs .monaco-findInput.disabled{background-color:#e1e1e1}.vs-dark .monaco-findInput.disabled{background-color:#333}.monaco-findInput.highlight-0 .controls,.hc-light .monaco-findInput.highlight-0 .controls{animation:monaco-findInput-highlight-0 .1s linear 0s}.monaco-findInput.highlight-1 .controls,.hc-light .monaco-findInput.highlight-1 .controls{animation:monaco-findInput-highlight-1 .1s linear 0s}.hc-black .monaco-findInput.highlight-0 .controls,.vs-dark .monaco-findInput.highlight-0 .controls{animation:monaco-findInput-highlight-dark-0 .1s linear 0s}.hc-black .monaco-findInput.highlight-1 .controls,.vs-dark .monaco-findInput.highlight-1 .controls{animation:monaco-findInput-highlight-dark-1 .1s linear 0s}@keyframes monaco-findInput-highlight-0{0%{background:#fdff00cc}to{background:transparent}}@keyframes monaco-findInput-highlight-1{0%{background:#fdff00cc}99%{background:transparent}}@keyframes monaco-findInput-highlight-dark-0{0%{background:#ffffff70}to{background:transparent}}@keyframes monaco-findInput-highlight-dark-1{0%{background:#ffffff70}99%{background:transparent}}.colorpicker-widget{height:190px;user-select:none;-webkit-user-select:none}.colorpicker-color-decoration,.hc-light .colorpicker-color-decoration{border:solid .1em #000;box-sizing:border-box;margin:.1em .2em 0;width:.8em;height:.8em;line-height:.8em;display:inline-block;cursor:pointer}.hc-black .colorpicker-color-decoration,.vs-dark .colorpicker-color-decoration{border:solid .1em #eee}.colorpicker-header{display:flex;height:24px;position:relative;background:url(data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAQAAAAECAYAAACp8Z5+AAAAAXNSR0IArs4c6QAAAARnQU1BAACxjwv8YQUAAAAJcEhZcwAADsMAAA7DAcdvqGQAAAAZdEVYdFNvZnR3YXJlAHBhaW50Lm5ldCA0LjAuMTZEaa/1AAAAHUlEQVQYV2PYvXu3JAi7uLiAMaYAjAGTQBPYLQkAa/0Zef3qRswAAAAASUVORK5CYII=);background-size:9px 9px;image-rendering:pixelated}.colorpicker-header .picked-color{width:240px;display:flex;align-items:center;justify-content:center;line-height:24px;cursor:pointer;color:#fff;flex:1;white-space:nowrap;overflow:hidden}.colorpicker-header .picked-color .picked-color-presentation{white-space:nowrap;margin-left:5px;margin-right:5px}.colorpicker-header .picked-color .codicon{color:inherit;font-size:14px}.colorpicker-header .picked-color.light{color:#000}.colorpicker-header .original-color{width:74px;z-index:inherit;cursor:pointer}.standalone-colorpicker{color:var(--vscode-editorHoverWidget-foreground);background-color:var(--vscode-editorHoverWidget-background);border:1px solid var(--vscode-editorHoverWidget-border)}.colorpicker-header.standalone-colorpicker{border-bottom:none}.colorpicker-header .close-button{cursor:pointer;background-color:var(--vscode-editorHoverWidget-background);border-left:1px solid var(--vscode-editorHoverWidget-border)}.colorpicker-header .close-button-inner-div{width:100%;height:100%;text-align:center}.colorpicker-header .close-button-inner-div:hover{background-color:var(--vscode-toolbar-hoverBackground)}.colorpicker-header .close-icon{padding:3px}.colorpicker-body{display:flex;padding:8px;position:relative}.colorpicker-body .saturation-wrap{overflow:hidden;height:150px;position:relative;min-width:220px;flex:1}.colorpicker-body .saturation-box{height:150px;position:absolute}.colorpicker-body .saturation-selection{width:9px;height:9px;margin:-5px 0 0 -5px;border:1px solid rgb(255,255,255);border-radius:100%;box-shadow:0 0 2px #000c;position:absolute}.colorpicker-body .strip{width:25px;height:150px}.colorpicker-body .standalone-strip{width:25px;height:122px}.colorpicker-body .hue-strip{position:relative;margin-left:8px;cursor:grab;background:linear-gradient(to bottom,red,#ff0 17%,#0f0 33%,#0ff,#00f 67%,#f0f 83%,red)}.colorpicker-body .opacity-strip{position:relative;margin-left:8px;cursor:grab;background:url(data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAQAAAAECAYAAACp8Z5+AAAAAXNSR0IArs4c6QAAAARnQU1BAACxjwv8YQUAAAAJcEhZcwAADsMAAA7DAcdvqGQAAAAZdEVYdFNvZnR3YXJlAHBhaW50Lm5ldCA0LjAuMTZEaa/1AAAAHUlEQVQYV2PYvXu3JAi7uLiAMaYAjAGTQBPYLQkAa/0Zef3qRswAAAAASUVORK5CYII=);background-size:9px 9px;image-rendering:pixelated}.colorpicker-body .strip.grabbing{cursor:grabbing}.colorpicker-body .slider{position:absolute;top:0;left:-2px;width:calc(100% + 4px);height:4px;box-sizing:border-box;border:1px solid rgba(255,255,255,.71);box-shadow:0 0 1px #000000d9}.colorpicker-body .strip .overlay{height:150px;pointer-events:none}.colorpicker-body .standalone-strip .standalone-overlay{height:122px;pointer-events:none}.standalone-colorpicker-body{display:block;border:1px solid transparent;border-bottom:1px solid var(--vscode-editorHoverWidget-border);overflow:hidden}.colorpicker-body .insert-button{position:absolute;height:20px;width:58px;padding:0;right:8px;bottom:8px;background:var(--vscode-button-background);color:var(--vscode-button-foreground);border-radius:2px;border:none;cursor:pointer}.colorpicker-body .insert-button:hover{background:var(--vscode-button-hoverBackground)}.monaco-editor .peekview-widget .head{box-sizing:border-box;display:flex;justify-content:space-between;flex-wrap:nowrap}.monaco-editor .peekview-widget .head .peekview-title{display:flex;align-items:baseline;font-size:13px;margin-left:20px;min-width:0;text-overflow:ellipsis;overflow:hidden}.monaco-editor .peekview-widget .head .peekview-title.clickable{cursor:pointer}.monaco-editor .peekview-widget .head .peekview-title .dirname:not(:empty){font-size:.9em;margin-left:.5em}.monaco-editor .peekview-widget .head .peekview-title .meta{white-space:nowrap;overflow:hidden;text-overflow:ellipsis}.monaco-editor .peekview-widget .head .peekview-title .dirname,.monaco-editor .peekview-widget .head .peekview-title .filename{overflow:hidden;text-overflow:ellipsis;white-space:nowrap}.monaco-editor .peekview-widget .head .peekview-title .meta:not(:empty):before{content:\"-\";padding:0 .3em}.monaco-editor .peekview-widget .head .peekview-actions{flex:1;text-align:right;padding-right:2px}.monaco-editor .peekview-widget .head .peekview-actions>.monaco-action-bar{display:inline-block}.monaco-editor .peekview-widget .head .peekview-actions>.monaco-action-bar,.monaco-editor .peekview-widget .head .peekview-actions>.monaco-action-bar>.actions-container{height:100%}.monaco-editor .peekview-widget>.body{border-top:1px solid;position:relative}.monaco-editor .peekview-widget .head .peekview-title .codicon{margin-right:4px;align-self:center}.monaco-editor .peekview-widget .monaco-list .monaco-list-row.focused .codicon{color:inherit!important}.monaco-editor .zone-widget{position:absolute;z-index:10}.monaco-editor .zone-widget .zone-widget-container{border-top-style:solid;border-bottom-style:solid;border-top-width:0;border-bottom-width:0;position:relative}.monaco-split-view2{position:relative;width:100%;height:100%}.monaco-split-view2>.sash-container{position:absolute;width:100%;height:100%;pointer-events:none}.monaco-split-view2>.sash-container>.monaco-sash{pointer-events:initial}.monaco-split-view2>.monaco-scrollable-element{width:100%;height:100%}.monaco-split-view2>.monaco-scrollable-element>.split-view-container{width:100%;height:100%;white-space:nowrap;position:relative}.monaco-split-view2>.monaco-scrollable-element>.split-view-container>.split-view-view{white-space:initial;position:absolute}.monaco-split-view2>.monaco-scrollable-element>.split-view-container>.split-view-view:not(.visible){display:none}.monaco-split-view2.vertical>.monaco-scrollable-element>.split-view-container>.split-view-view{width:100%}.monaco-split-view2.horizontal>.monaco-scrollable-element>.split-view-container>.split-view-view{height:100%}.monaco-split-view2.separator-border>.monaco-scrollable-element>.split-view-container>.split-view-view:not(:first-child):before{content:\" \";position:absolute;top:0;left:0;z-index:5;pointer-events:none;background-color:var(--separator-border)}.monaco-split-view2.separator-border.horizontal>.monaco-scrollable-element>.split-view-container>.split-view-view:not(:first-child):before{height:100%;width:1px}.monaco-split-view2.separator-border.vertical>.monaco-scrollable-element>.split-view-container>.split-view-view:not(:first-child):before{height:1px;width:100%}.monaco-table{display:flex;flex-direction:column;position:relative;height:100%;width:100%;white-space:nowrap;overflow:hidden}.monaco-table>.monaco-split-view2{border-bottom:1px solid transparent}.monaco-table>.monaco-list{flex:1}.monaco-table-tr{display:flex;height:100%}.monaco-table-th{width:100%;height:100%;font-weight:700;overflow:hidden;text-overflow:ellipsis}.monaco-table-th,.monaco-table-td{box-sizing:border-box;flex-shrink:0;overflow:hidden;white-space:nowrap;text-overflow:ellipsis}.monaco-table>.monaco-split-view2 .monaco-sash.vertical:before{content:\"\";position:absolute;left:calc(var(--vscode-sash-size) / 2);width:0;border-left:1px solid transparent}.monaco-enable-motion .monaco-table>.monaco-split-view2,.monaco-enable-motion .monaco-table>.monaco-split-view2 .monaco-sash.vertical:before{transition:border-color .2s ease-out}.monaco-tl-row{display:flex;height:100%;align-items:center;position:relative}.monaco-tl-row.disabled{cursor:default}.monaco-tl-indent{height:100%;position:absolute;top:0;left:16px;pointer-events:none}.hide-arrows .monaco-tl-indent{left:12px}.monaco-tl-indent>.indent-guide{display:inline-block;box-sizing:border-box;height:100%;border-left:1px solid transparent;opacity:0}.monaco-enable-motion .monaco-tl-indent>.indent-guide{transition:opacity .1s linear}.monaco-tl-twistie,.monaco-tl-contents{height:100%}.monaco-tl-twistie{font-size:10px;text-align:right;padding-right:6px;flex-shrink:0;width:16px;display:flex!important;align-items:center;justify-content:center;transform:translate(3px)}.monaco-tl-contents{flex:1;overflow:hidden}.monaco-tl-twistie:before{border-radius:20px}.monaco-tl-twistie.collapsed:before{transform:rotate(-90deg)}.monaco-tl-twistie.codicon-tree-item-loading:before{animation:codicon-spin 1.25s steps(30) infinite}.monaco-tree-type-filter{position:absolute;top:0;right:0;display:flex;padding:3px;max-width:200px;z-index:100;margin:0 10px 0 6px;border:1px solid var(--vscode-widget-border);border-bottom-left-radius:4px;border-bottom-right-radius:4px}.monaco-enable-motion .monaco-tree-type-filter{transition:top .3s}.monaco-tree-type-filter.disabled{top:-40px!important}.monaco-tree-type-filter-input{flex:1}.monaco-tree-type-filter-input .monaco-inputbox{height:23px}.monaco-tree-type-filter-input .monaco-inputbox>.ibwrapper>.input,.monaco-tree-type-filter-input .monaco-inputbox>.ibwrapper>.mirror{padding:2px 4px}.monaco-tree-type-filter-input .monaco-findInput>.controls{top:2px}.monaco-tree-type-filter-actionbar{margin-left:4px}.monaco-tree-type-filter-actionbar .monaco-action-bar .action-label{padding:2px}.monaco-list .monaco-scrollable-element .monaco-tree-sticky-container{position:absolute;top:0;left:0;width:100%;height:0;z-index:13;background-color:var(--vscode-sideBar-background)}.monaco-list .monaco-scrollable-element .monaco-tree-sticky-container .monaco-tree-sticky-row.monaco-list-row{position:absolute;width:100%;opacity:1!important;overflow:hidden;background-color:var(--vscode-sideBar-background)}.monaco-list .monaco-scrollable-element .monaco-tree-sticky-container .monaco-tree-sticky-row:hover{background-color:var(--vscode-list-hoverBackground)!important;cursor:pointer}.monaco-list .monaco-scrollable-element .monaco-tree-sticky-container.empty,.monaco-list .monaco-scrollable-element .monaco-tree-sticky-container.empty .monaco-tree-sticky-container-shadow{display:none}.monaco-list .monaco-scrollable-element .monaco-tree-sticky-container .monaco-tree-sticky-container-shadow{position:absolute;bottom:-3px;left:0;height:0px;width:100%}.monaco-list .monaco-scrollable-element .monaco-tree-sticky-container[tabindex=\"0\"]:focus{outline:none}.monaco-editor .zone-widget .zone-widget-container.reference-zone-widget{border-top-width:1px;border-bottom-width:1px}.monaco-editor .reference-zone-widget .inline{display:inline-block;vertical-align:top}.monaco-editor .reference-zone-widget .messages{height:100%;width:100%;text-align:center;padding:3em 0}.monaco-editor .reference-zone-widget .ref-tree{line-height:23px;background-color:var(--vscode-peekViewResult-background);color:var(--vscode-peekViewResult-lineForeground)}.monaco-editor .reference-zone-widget .ref-tree .reference{text-overflow:ellipsis;overflow:hidden}.monaco-editor .reference-zone-widget .ref-tree .reference-file{display:inline-flex;width:100%;height:100%;color:var(--vscode-peekViewResult-fileForeground)}.monaco-editor .reference-zone-widget .ref-tree .monaco-list:focus .selected .reference-file{color:inherit!important}.monaco-editor .reference-zone-widget .ref-tree .monaco-list:focus .monaco-list-rows>.monaco-list-row.selected:not(.highlighted){background-color:var(--vscode-peekViewResult-selectionBackground);color:var(--vscode-peekViewResult-selectionForeground)!important}.monaco-editor .reference-zone-widget .ref-tree .reference-file .count{margin-right:12px;margin-left:auto}.monaco-editor .reference-zone-widget .ref-tree .referenceMatch .highlight{color:var(--vscode-peekViewResult-fileForeground)!important;background-color:var(--vscode-peekViewResult-matchHighlightBackground)!important}.monaco-editor .reference-zone-widget .preview .reference-decoration{background-color:var(--vscode-peekViewEditor-matchHighlightBackground);border:2px solid var(--vscode-peekViewEditor-matchHighlightBorder);box-sizing:border-box}.monaco-editor .reference-zone-widget .preview .monaco-editor .monaco-editor-background,.monaco-editor .reference-zone-widget .preview .monaco-editor .inputarea.ime-input{background-color:var(--vscode-peekViewEditor-background)}.monaco-editor .reference-zone-widget .preview .monaco-editor .margin{background-color:var(--vscode-peekViewEditorGutter-background)}.monaco-editor.hc-black .reference-zone-widget .ref-tree .reference-file,.monaco-editor.hc-light .reference-zone-widget .ref-tree .reference-file{font-weight:700}.monaco-editor.hc-black .reference-zone-widget .ref-tree .referenceMatch .highlight,.monaco-editor.hc-light .reference-zone-widget .ref-tree .referenceMatch .highlight{border:1px dotted var(--vscode-contrastActiveBorder, transparent);box-sizing:border-box}.monaco-count-badge{padding:3px 5px;border-radius:11px;font-size:11px;min-width:18px;min-height:18px;line-height:11px;font-weight:400;text-align:center;display:inline-block;box-sizing:border-box}.monaco-count-badge.long{padding:2px 3px;border-radius:2px;min-height:auto;line-height:normal}.monaco-icon-label{display:flex;overflow:hidden;text-overflow:ellipsis}.monaco-icon-label:before{background-size:16px;background-position:left center;background-repeat:no-repeat;padding-right:6px;width:16px;height:22px;line-height:inherit!important;display:inline-block;-webkit-font-smoothing:antialiased;-moz-osx-font-smoothing:grayscale;vertical-align:top;flex-shrink:0}.monaco-icon-label-iconpath{width:16px;height:22px;margin-right:6px;display:flex}.monaco-icon-label-container.disabled{color:var(--vscode-disabledForeground)}.monaco-icon-label>.monaco-icon-label-container{min-width:0;overflow:hidden;text-overflow:ellipsis;flex:1}.monaco-icon-label>.monaco-icon-label-container>.monaco-icon-name-container>.label-name{color:inherit;white-space:pre}.monaco-icon-label>.monaco-icon-label-container>.monaco-icon-name-container>.label-name>.label-separator{margin:0 2px;opacity:.5}.monaco-icon-label>.monaco-icon-label-container>.monaco-icon-suffix-container>.label-suffix{opacity:.7;white-space:pre}.monaco-icon-label>.monaco-icon-label-container>.monaco-icon-description-container>.label-description{opacity:.7;margin-left:.5em;font-size:.9em;white-space:pre}.monaco-icon-label.nowrap>.monaco-icon-label-container>.monaco-icon-description-container>.label-description{white-space:nowrap}.vs .monaco-icon-label>.monaco-icon-label-container>.monaco-icon-description-container>.label-description{opacity:.95}.monaco-icon-label.bold>.monaco-icon-label-container>.monaco-icon-name-container>.label-name,.monaco-icon-label.bold>.monaco-icon-label-container>.monaco-icon-description-container>.label-description{font-weight:700}.monaco-icon-label.italic>.monaco-icon-label-container>.monaco-icon-name-container>.label-name,.monaco-icon-label.italic>.monaco-icon-label-container>.monaco-icon-description-container>.label-description{font-style:italic}.monaco-icon-label.deprecated{text-decoration:line-through;opacity:.66}.monaco-icon-label.strikethrough>.monaco-icon-label-container>.monaco-icon-name-container>.label-name,.monaco-icon-label.strikethrough>.monaco-icon-label-container>.monaco-icon-description-container>.label-description{text-decoration:line-through}.monaco-icon-label:after{opacity:.75;font-size:90%;font-weight:600;margin:auto 16px 0 5px;text-align:center}.monaco-list:focus .selected .monaco-icon-label,.monaco-list:focus .selected .monaco-icon-label:after{color:inherit!important}.monaco-list-row.focused.selected .label-description,.monaco-list-row.selected .label-description{opacity:.8}.monaco-editor .peekview-widget .head .peekview-title .severity-icon{display:inline-block;vertical-align:text-top;margin-right:4px}.monaco-editor .marker-widget{text-overflow:ellipsis;white-space:nowrap}.monaco-editor .marker-widget>.stale{opacity:.6;font-style:italic}.monaco-editor .marker-widget .title{display:inline-block;padding-right:5px}.monaco-editor .marker-widget .descriptioncontainer{position:absolute;white-space:pre;user-select:text;-webkit-user-select:text;padding:8px 12px 0 20px}.monaco-editor .marker-widget .descriptioncontainer .message{display:flex;flex-direction:column}.monaco-editor .marker-widget .descriptioncontainer .message .details{padding-left:6px}.monaco-editor .marker-widget .descriptioncontainer .message .source,.monaco-editor .marker-widget .descriptioncontainer .message span.code{opacity:.6}.monaco-editor .marker-widget .descriptioncontainer .message a.code-link{opacity:.6;color:inherit}.monaco-editor .marker-widget .descriptioncontainer .message a.code-link:before{content:\"(\"}.monaco-editor .marker-widget .descriptioncontainer .message a.code-link:after{content:\")\"}.monaco-editor .marker-widget .descriptioncontainer .message a.code-link>span{text-decoration:underline;border-bottom:1px solid transparent;text-underline-position:under;color:var(--vscode-textLink-activeForeground)}.monaco-editor .marker-widget .descriptioncontainer .filename{cursor:pointer;color:var(--vscode-textLink-activeForeground)}.monaco-editor .zone-widget .codicon.codicon-error,.markers-panel .marker-icon.error,.markers-panel .marker-icon .codicon.codicon-error,.text-search-provider-messages .providerMessage .codicon.codicon-error,.extensions-viewlet>.extensions .codicon.codicon-error,.extension-editor .codicon.codicon-error,.chat-attached-context-attachment .codicon.codicon-error{color:var(--vscode-problemsErrorIcon-foreground)}.monaco-editor .zone-widget .codicon.codicon-warning,.markers-panel .marker-icon.warning,.markers-panel .marker-icon .codicon.codicon-warning,.text-search-provider-messages .providerMessage .codicon.codicon-warning,.extensions-viewlet>.extensions .codicon.codicon-warning,.extension-editor .codicon.codicon-warning,.preferences-editor .codicon.codicon-warning{color:var(--vscode-problemsWarningIcon-foreground)}.monaco-editor .zone-widget .codicon.codicon-info,.markers-panel .marker-icon.info,.markers-panel .marker-icon .codicon.codicon-info,.text-search-provider-messages .providerMessage .codicon.codicon-info,.extensions-viewlet>.extensions .codicon.codicon-info,.extension-editor .codicon.codicon-info{color:var(--vscode-problemsInfoIcon-foreground)}.monaco-editor .hoverHighlight{background-color:var(--vscode-editor-hoverHighlightBackground)}.monaco-editor .monaco-resizable-hover{border:1px solid var(--vscode-editorHoverWidget-border);border-radius:3px;box-sizing:content-box}.monaco-editor .monaco-resizable-hover>.monaco-hover{border:none;border-radius:none}.monaco-editor .monaco-hover{border:1px solid var(--vscode-editorHoverWidget-border);border-radius:3px;color:var(--vscode-editorHoverWidget-foreground);background-color:var(--vscode-editorHoverWidget-background)}.monaco-editor .monaco-hover a{color:var(--vscode-textLink-foreground)}.monaco-editor .monaco-hover a:hover{color:var(--vscode-textLink-activeForeground)}.monaco-editor .monaco-hover .hover-row{display:flex}.monaco-editor .monaco-hover .hover-row.hover-row-with-copy{position:relative;padding-right:20px}.monaco-editor .monaco-hover .hover-row .hover-row-contents{min-width:0;display:flex;flex-direction:column}.monaco-editor .monaco-hover .hover-row .verbosity-actions{border-right:1px solid var(--vscode-editorHoverWidget-border);width:22px;overflow-y:clip}.monaco-editor .monaco-hover .hover-row .verbosity-actions-inner{display:flex;flex-direction:column;padding-left:5px;padding-right:5px;justify-content:flex-end;position:relative}.monaco-editor .monaco-hover .hover-row .verbosity-actions-inner .codicon{cursor:pointer;font-size:11px}.monaco-editor .monaco-hover .hover-row .verbosity-actions-inner .codicon.enabled{color:var(--vscode-textLink-foreground)}.monaco-editor .monaco-hover .hover-row .verbosity-actions-inner .codicon.disabled{opacity:.6}.monaco-editor .monaco-hover .hover-row .actions{background-color:var(--vscode-editorHoverWidget-statusBarBackground)}.monaco-editor .monaco-hover code{background-color:var(--vscode-textCodeBlock-background)}.monaco-editor .monaco-hover .hover-copy-button{position:absolute;top:4px;right:4px;padding:2px 4px;border-radius:3px;display:flex;align-items:center;justify-content:center;opacity:0}.monaco-editor .monaco-hover .hover-row-with-copy:hover .hover-copy-button,.monaco-editor .monaco-hover .hover-row-with-copy:focus-within .hover-copy-button{opacity:1}.monaco-editor .monaco-hover .hover-copy-button:hover{background-color:var(--vscode-toolbar-hoverBackground);cursor:pointer}.monaco-editor .monaco-hover .hover-copy-button:focus{outline:1px solid var(--vscode-focusBorder);outline-offset:-1px}.monaco-editor .monaco-hover .hover-copy-button .codicon{font-size:16px;color:var(--vscode-foreground)}.monaco-editor.vs .dnd-target,.monaco-editor.hc-light .dnd-target{border-right:2px dotted black;color:#fff}.monaco-editor.vs-dark .dnd-target{border-right:2px dotted #AEAFAD;color:#51504f}.monaco-editor.hc-black .dnd-target{border-right:2px dotted #fff;color:#000}.monaco-editor.mouse-default .view-lines,.monaco-editor.vs-dark.mac.mouse-default .view-lines,.monaco-editor.hc-black.mac.mouse-default .view-lines,.monaco-editor.hc-light.mac.mouse-default .view-lines{cursor:default}.monaco-editor.mouse-copy .view-lines,.monaco-editor.vs-dark.mac.mouse-copy .view-lines,.monaco-editor.hc-black.mac.mouse-copy .view-lines,.monaco-editor.hc-light.mac.mouse-copy .view-lines{cursor:copy}.monaco-editor .findOptionsWidget{background-color:var(--vscode-editorWidget-background);color:var(--vscode-editorWidget-foreground);box-shadow:0 0 8px 2px var(--vscode-widget-shadow);border:2px solid var(--vscode-contrastBorder)}.monaco-editor .margin-view-overlays .codicon-folding-manual-collapsed,.monaco-editor .margin-view-overlays .codicon-folding-manual-expanded,.monaco-editor .margin-view-overlays .codicon-folding-expanded,.monaco-editor .margin-view-overlays .codicon-folding-collapsed{cursor:pointer;opacity:0;transition:opacity .5s;display:flex;align-items:center;justify-content:center;font-size:140%;margin-left:2px}.monaco-reduce-motion .monaco-editor .margin-view-overlays .codicon-folding-manual-collapsed,.monaco-reduce-motion .monaco-editor .margin-view-overlays .codicon-folding-manual-expanded,.monaco-reduce-motion .monaco-editor .margin-view-overlays .codicon-folding-expanded,.monaco-reduce-motion .monaco-editor .margin-view-overlays .codicon-folding-collapsed{transition:initial}.monaco-editor .margin-view-overlays:hover .codicon,.monaco-editor .margin-view-overlays .codicon.codicon-folding-collapsed,.monaco-editor .margin-view-overlays .codicon.codicon-folding-manual-collapsed,.monaco-editor .margin-view-overlays .codicon.alwaysShowFoldIcons{opacity:1}.monaco-editor .inline-folded:after{color:var(--vscode-editor-foldPlaceholderForeground);margin:.1em .2em 0;content:\"⋯\";display:inline;line-height:1em;cursor:pointer}.monaco-editor .folded-background{background-color:var(--vscode-editor-foldBackground)}.monaco-editor .cldr.codicon.codicon-folding-expanded,.monaco-editor .cldr.codicon.codicon-folding-collapsed,.monaco-editor .cldr.codicon.codicon-folding-manual-expanded,.monaco-editor .cldr.codicon.codicon-folding-manual-collapsed{color:var(--vscode-editorGutter-foldingControlForeground)!important}.monaco-editor .snippet-placeholder{min-width:2px;outline-style:solid;outline-width:1px;background-color:var(--vscode-editor-snippetTabstopHighlightBackground, transparent);outline-color:var(--vscode-editor-snippetTabstopHighlightBorder, transparent)}.monaco-editor .finish-snippet-placeholder{outline-style:solid;outline-width:1px;background-color:var(--vscode-editor-snippetFinalTabstopHighlightBackground, transparent);outline-color:var(--vscode-editor-snippetFinalTabstopHighlightBorder, transparent)}.monaco-editor .suggest-widget{width:430px;z-index:40;display:flex;flex-direction:column;border-radius:3px}.monaco-editor .suggest-widget.message{flex-direction:row;align-items:center}.monaco-editor .suggest-widget,.monaco-editor .suggest-details{flex:0 1 auto;width:100%;border-style:solid;border-width:1px;border-color:var(--vscode-editorSuggestWidget-border);background-color:var(--vscode-editorSuggestWidget-background)}.monaco-editor.hc-black .suggest-widget,.monaco-editor.hc-black .suggest-details,.monaco-editor.hc-light .suggest-widget,.monaco-editor.hc-light .suggest-details{border-width:2px}.monaco-editor .suggest-widget .suggest-status-bar{box-sizing:border-box;display:none;flex-flow:row nowrap;justify-content:space-between;width:100%;font-size:80%;padding:0 4px;border-top:1px solid var(--vscode-editorSuggestWidget-border);overflow:hidden}.monaco-editor .suggest-widget.with-status-bar .suggest-status-bar{display:flex}.monaco-editor .suggest-widget .suggest-status-bar .left{padding-right:8px}.monaco-editor .suggest-widget.with-status-bar .suggest-status-bar .action-label{color:var(--vscode-editorSuggestWidgetStatus-foreground)}.monaco-editor .suggest-widget.with-status-bar .suggest-status-bar .action-item:not(:last-of-type) .action-label{margin-right:0}.monaco-editor .suggest-widget.with-status-bar .suggest-status-bar .action-item:not(:last-of-type) .action-label:after{content:\", \";margin-right:.3em}.monaco-editor .suggest-widget.with-status-bar .monaco-list .monaco-list-row>.contents>.main>.right>.readMore,.monaco-editor .suggest-widget.with-status-bar .monaco-list .monaco-list-row.focused.string-label>.contents>.main>.right>.readMore{display:none}.monaco-editor .suggest-widget.with-status-bar:not(.docs-side) .monaco-list .monaco-list-row:hover>.contents>.main>.right.can-expand-details>.details-label{width:100%}.monaco-editor .suggest-widget>.message{padding-left:22px}.monaco-editor .suggest-widget>.tree{height:100%;width:100%}.monaco-editor .suggest-widget .monaco-list{user-select:none;-webkit-user-select:none}.monaco-editor .suggest-widget .monaco-list .monaco-list-row{display:flex;-mox-box-sizing:border-box;box-sizing:border-box;padding-right:10px;background-repeat:no-repeat;background-position:2px 2px;white-space:nowrap;cursor:pointer;touch-action:none}.monaco-editor .suggest-widget .monaco-list .monaco-list-row.focused{color:var(--vscode-editorSuggestWidget-selectedForeground)}.monaco-editor .suggest-widget .monaco-list .monaco-list-row.focused .codicon{color:var(--vscode-editorSuggestWidget-selectedIconForeground)}.monaco-editor .suggest-widget .monaco-list .monaco-list-row>.contents{flex:1;height:100%;overflow:hidden;padding-left:2px}.monaco-editor .suggest-widget .monaco-list .monaco-list-row>.contents>.main{display:flex;overflow:hidden;text-overflow:ellipsis;white-space:pre;justify-content:space-between}.monaco-editor .suggest-widget .monaco-list .monaco-list-row>.contents>.main>.left,.monaco-editor .suggest-widget .monaco-list .monaco-list-row>.contents>.main>.right{display:flex}.monaco-editor .suggest-widget .monaco-list .monaco-list-row:not(.focused)>.contents>.main .monaco-icon-label{color:var(--vscode-editorSuggestWidget-foreground)}.monaco-editor .suggest-widget:not(.frozen) .monaco-highlighted-label .highlight{font-weight:700}.monaco-editor .suggest-widget .monaco-list .monaco-list-row>.contents>.main .monaco-highlighted-label .highlight{color:var(--vscode-editorSuggestWidget-highlightForeground)}.monaco-editor .suggest-widget .monaco-list .monaco-list-row.focused>.contents>.main .monaco-highlighted-label .highlight{color:var(--vscode-editorSuggestWidget-focusHighlightForeground)}.monaco-editor .suggest-details>.monaco-scrollable-element>.body>.header>.codicon-close,.monaco-editor .suggest-widget .monaco-list .monaco-list-row>.contents>.main>.right>.readMore:before{color:inherit;opacity:1;font-size:14px;cursor:pointer}.monaco-editor .suggest-details>.monaco-scrollable-element>.body>.header>.codicon-close{position:absolute;top:6px;right:2px}.monaco-editor .suggest-details>.monaco-scrollable-element>.body>.header>.codicon-close:hover,.monaco-editor .suggest-widget .monaco-list .monaco-list-row>.contents>.main>.right>.readMore:hover{opacity:1}.monaco-editor .suggest-widget .monaco-list .monaco-list-row>.contents>.main>.right>.details-label{opacity:.7}.monaco-editor .suggest-widget .monaco-list .monaco-list-row>.contents>.main>.left>.signature-label{overflow:hidden;text-overflow:ellipsis;opacity:.6}.monaco-editor .suggest-widget .monaco-list .monaco-list-row>.contents>.main>.left>.qualifier-label{margin-left:12px;opacity:.4;font-size:85%;line-height:initial;text-overflow:ellipsis;overflow:hidden;align-self:center}.monaco-editor .suggest-widget .monaco-list .monaco-list-row>.contents>.main>.right>.details-label{font-size:85%;margin-left:1.1em;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}.monaco-editor .suggest-widget .monaco-list .monaco-list-row>.contents>.main>.right>.details-label>.monaco-tokenized-source{display:inline}.monaco-editor .suggest-widget .monaco-list .monaco-list-row>.contents>.main>.right>.details-label{display:none}.monaco-editor .suggest-widget:not(.shows-details) .monaco-list .monaco-list-row.focused>.contents>.main>.right>.details-label{display:inline}.monaco-editor .suggest-widget .monaco-list .monaco-list-row:not(.string-label)>.contents>.main>.right>.details-label,.monaco-editor .suggest-widget.docs-side .monaco-list .monaco-list-row.focused:not(.string-label)>.contents>.main>.right>.details-label{display:inline}.monaco-editor .suggest-widget:not(.docs-side) .monaco-list .monaco-list-row.focused:hover>.contents>.main>.right.can-expand-details>.details-label{width:calc(100% - 26px)}.monaco-editor .suggest-widget .monaco-list .monaco-list-row>.contents>.main>.left{flex-shrink:1;flex-grow:1;overflow:hidden}.monaco-editor .suggest-widget .monaco-list .monaco-list-row>.contents>.main>.left>.monaco-icon-label{flex-shrink:0}.monaco-editor .suggest-widget .monaco-list .monaco-list-row:not(.string-label)>.contents>.main>.left>.monaco-icon-label{max-width:100%}.monaco-editor .suggest-widget .monaco-list .monaco-list-row.string-label>.contents>.main>.left>.monaco-icon-label{flex-shrink:1}.monaco-editor .suggest-widget .monaco-list .monaco-list-row>.contents>.main>.right{overflow:hidden;flex-shrink:4;max-width:70%}.monaco-editor .suggest-widget .monaco-list .monaco-list-row>.contents>.main>.right>.readMore{display:inline-block;position:absolute;right:10px;width:18px;height:18px;visibility:hidden}.monaco-editor .suggest-widget.docs-side .monaco-list .monaco-list-row>.contents>.main>.right>.readMore{display:none!important}.monaco-editor .suggest-widget .monaco-list .monaco-list-row.string-label>.contents>.main>.right>.readMore{display:none}.monaco-editor .suggest-widget .monaco-list .monaco-list-row.focused.string-label>.contents>.main>.right>.readMore{display:inline-block}.monaco-editor .suggest-widget .monaco-list .monaco-list-row.focused:hover>.contents>.main>.right>.readMore{visibility:visible}.monaco-editor .suggest-widget .monaco-list .monaco-list-row .monaco-icon-label.deprecated{opacity:.66;text-decoration:unset}.monaco-editor .suggest-widget .monaco-list .monaco-list-row .monaco-icon-label.deprecated>.monaco-icon-label-container>.monaco-icon-name-container{text-decoration:line-through}.monaco-editor .suggest-widget .monaco-list .monaco-list-row .monaco-icon-label:before{height:100%}.monaco-editor .suggest-widget .monaco-list .monaco-list-row .icon{display:block;height:16px;width:16px;margin-left:2px;background-repeat:no-repeat;background-size:80%;background-position:center}.monaco-editor .suggest-widget .monaco-list .monaco-list-row .icon.hide{display:none}.monaco-editor .suggest-widget .monaco-list .monaco-list-row .suggest-icon{display:flex;align-items:center;margin-right:4px}.monaco-editor .suggest-widget.no-icons .monaco-list .monaco-list-row .icon,.monaco-editor .suggest-widget.no-icons .monaco-list .monaco-list-row .suggest-icon:before{display:none}.monaco-editor .suggest-widget .monaco-list .monaco-list-row .icon.customcolor .colorspan{margin:0 0 0 .3em;border:.1em solid #000;width:.7em;height:.7em;display:inline-block}.monaco-editor .suggest-details-container{z-index:41}.monaco-editor .suggest-details{display:flex;flex-direction:column;cursor:default;color:var(--vscode-editorSuggestWidget-foreground)}.monaco-editor .suggest-details:focus{border-color:var(--vscode-focusBorder)}.monaco-editor .suggest-details a{color:var(--vscode-textLink-foreground)}.monaco-editor .suggest-details a:hover{color:var(--vscode-textLink-activeForeground)}.monaco-editor .suggest-details code{background-color:var(--vscode-textCodeBlock-background)}.monaco-editor .suggest-details.no-docs{display:none}.monaco-editor .suggest-details>.monaco-scrollable-element{flex:1}.monaco-editor .suggest-details>.monaco-scrollable-element>.body{box-sizing:border-box;height:100%;width:100%}.monaco-editor .suggest-details>.monaco-scrollable-element>.body>.header>.type{flex:2;overflow:hidden;text-overflow:ellipsis;opacity:.7;white-space:pre;margin:0 24px 0 0;padding:4px 0 4px 5px}.monaco-editor .suggest-details.detail-and-doc>.monaco-scrollable-element>.body>.header>.type{padding-bottom:12px}.monaco-editor .suggest-details>.monaco-scrollable-element>.body>.header>.type.auto-wrap{white-space:normal;word-break:break-all}.monaco-editor .suggest-details>.monaco-scrollable-element>.body>.docs{margin:0;padding:4px 5px;white-space:pre-wrap}.monaco-editor .suggest-details.no-type>.monaco-scrollable-element>.body>.docs{margin-right:24px;overflow:hidden}.monaco-editor .suggest-details>.monaco-scrollable-element>.body>.docs.markdown-docs{padding:0;white-space:initial;min-height:calc(1rem + 8px)}.monaco-editor .suggest-details>.monaco-scrollable-element>.body>.docs.markdown-docs>div,.monaco-editor .suggest-details>.monaco-scrollable-element>.body>.docs.markdown-docs>span:not(:empty){padding:4px 5px}.monaco-editor .suggest-details>.monaco-scrollable-element>.body>.docs.markdown-docs>div>p:first-child{margin-top:0}.monaco-editor .suggest-details>.monaco-scrollable-element>.body>.docs.markdown-docs>div>p:last-child{margin-bottom:0}.monaco-editor .suggest-details>.monaco-scrollable-element>.body>.docs.markdown-docs .monaco-tokenized-source{white-space:pre}.monaco-editor .suggest-details>.monaco-scrollable-element>.body>.docs .code{white-space:pre-wrap;word-wrap:break-word}.monaco-editor .suggest-details>.monaco-scrollable-element>.body>.docs.markdown-docs .codicon{vertical-align:sub}.monaco-editor .suggest-details>.monaco-scrollable-element>.body>p:empty{display:none}.monaco-editor .suggest-details code{border-radius:3px;padding:0 .4em}.monaco-editor .suggest-details ul,.monaco-editor .suggest-details ol{padding-left:20px}.monaco-editor .suggest-details p code{font-family:var(--monaco-monospace-font)}.monaco-editor .suggest-preview-additional-widget{white-space:nowrap}.monaco-editor .suggest-preview-additional-widget .content-spacer{color:transparent;white-space:pre}.monaco-editor .suggest-preview-additional-widget .button{display:inline-block;cursor:pointer;text-decoration:underline;text-underline-position:under}.monaco-editor .ghost-text-hidden{opacity:0;font-size:0}.monaco-editor .ghost-text-decoration,.monaco-editor .suggest-preview-text .ghost-text{font-style:italic}.monaco-editor .suggest-preview-text.clickable .view-line{z-index:1}.monaco-editor .ghost-text-decoration.clickable,.monaco-editor .ghost-text-decoration-preview.clickable,.monaco-editor .suggest-preview-text.clickable .ghost-text{cursor:pointer}.monaco-editor .inline-completion-text-to-replace{text-decoration:underline;text-underline-position:under}.monaco-editor .ghost-text-decoration,.monaco-editor .ghost-text-decoration-preview,.monaco-editor .suggest-preview-text .ghost-text{&.syntax-highlighted{opacity:.7}&:not(.syntax-highlighted){color:var(--vscode-editorGhostText-foreground)}background-color:var(--vscode-editorGhostText-background);border:1px solid var(--vscode-editorGhostText-border)}.monaco-editor .ghost-text-decoration.warning,.monaco-editor .ghost-text-decoration-preview.warning,.monaco-editor .suggest-preview-text .ghost-text.warning{background:var(--monaco-editor-warning-decoration) repeat-x bottom left;border-bottom:4px double var(--vscode-editorWarning-border)}.ghost-text-view-warning-widget-icon{.codicon{color:var(--vscode-editorWarning-foreground)!important}}.monaco-editor{.edits-fadeout-decoration{opacity:var(--animation-opacity, 1);background-color:var(--vscode-inlineEdit-modifiedChangedTextBackground)}}.monaco-editor .sticky-widget{overflow:hidden;border-bottom:1px solid var(--vscode-editorStickyScroll-border);width:100%;box-shadow:var(--vscode-editorStickyScroll-shadow) 0 4px 2px -2px;z-index:4;right:initial!important;margin-left:\"0px\"}.monaco-editor .sticky-widget .sticky-widget-line-numbers{float:left;background-color:var(--vscode-editorStickyScrollGutter-background)}.monaco-editor .sticky-widget.peek .sticky-widget-line-numbers{background-color:var(--vscode-peekViewEditorStickyScrollGutter-background)}.monaco-editor .sticky-widget .sticky-widget-lines-scrollable{display:inline-block;position:absolute;overflow:hidden;width:var(--vscode-editorStickyScroll-scrollableWidth);background-color:var(--vscode-editorStickyScroll-background)}.monaco-editor .sticky-widget.peek .sticky-widget-lines-scrollable{background-color:var(--vscode-peekViewEditorStickyScroll-background)}.monaco-editor .sticky-widget .sticky-widget-lines{position:absolute;background-color:inherit}.monaco-editor .sticky-widget .sticky-line-number,.monaco-editor .sticky-widget .sticky-line-content{color:var(--vscode-editorLineNumber-foreground);white-space:nowrap;display:inline-block;position:absolute;background-color:inherit}.monaco-editor .sticky-widget .sticky-line-number .codicon-folding-expanded,.monaco-editor .sticky-widget .sticky-line-number .codicon-folding-collapsed{float:right;transition:var(--vscode-editorStickyScroll-foldingOpacityTransition);position:absolute;margin-left:2px}.monaco-editor .sticky-widget .sticky-line-content{width:var(--vscode-editorStickyScroll-scrollableWidth);background-color:inherit;white-space:nowrap}.monaco-editor .sticky-widget .sticky-line-number-inner{display:inline-block;text-align:right}.monaco-editor .sticky-widget .sticky-line-content:hover{background-color:var(--vscode-editorStickyScrollHover-background);cursor:pointer}.monaco-editor{.inline-edits-view-indicator{display:flex;z-index:34;height:20px;color:var(--vscode-inlineEdit-gutterIndicator-primaryForeground);background-color:var(--vscode-inlineEdit-gutterIndicator-background);border:1px solid var(--vscode-inlineEdit-gutterIndicator-primaryBorder);border-radius:3px;align-items:center;padding:2px 10px 2px 2px;margin:0 4px;opacity:0;&.contained{transition:opacity .2s ease-in-out;transition-delay:.4s}&.visible{opacity:1}&.top{opacity:1;.icon{transform:rotate(90deg)}}&.bottom{opacity:1;.icon{transform:rotate(-90deg)}}.icon{display:flex;align-items:center;margin:0 2px;transform:none;transition:transform .2s ease-in-out;.codicon{color:var(--vscode-inlineEdit-gutterIndicator-primaryForeground)}}.label{margin:0 2px;display:flex;justify-content:center;width:100%}}.inline-edits-view .editorContainer{.preview .monaco-editor{.view-overlays .current-line-exact,.current-line-margin{border:none}}.inline-edits-view-zone.diagonal-fill{opacity:.5}}.strike-through{text-decoration:line-through}.inlineCompletions-line-insert{background:var(--vscode-inlineEdit-modifiedChangedLineBackground)}.inlineCompletions-line-delete{background:var(--vscode-inlineEdit-originalChangedLineBackground)}.inlineCompletions-char-insert{background:var(--vscode-inlineEdit-modifiedChangedTextBackground);cursor:pointer}.inlineCompletions-char-delete{background:var(--vscode-inlineEdit-originalChangedTextBackground)}.inlineCompletions-char-delete.diff-range-empty{margin-left:-1px;border-left:solid var(--vscode-inlineEdit-originalChangedTextBackground) 3px}.inlineCompletions-char-insert.diff-range-empty{border-left:solid var(--vscode-inlineEdit-modifiedChangedTextBackground) 3px}.inlineCompletions-char-delete.single-line-inline{border:1px solid var(--vscode-editorHoverWidget-border);margin:-2px 0 0 -2px}.inlineCompletions-char-insert.single-line-inline{border-top:1px solid var(--vscode-inlineEdit-modifiedBorder);border-bottom:1px solid var(--vscode-inlineEdit-modifiedBorder)}.inlineCompletions-char-insert.single-line-inline.start{border-top-left-radius:4px;border-bottom-left-radius:4px;border-left:1px solid var(--vscode-inlineEdit-modifiedBorder)}.inlineCompletions-char-insert.single-line-inline.end{border-top-right-radius:4px;border-bottom-right-radius:4px;border-right:1px solid var(--vscode-inlineEdit-modifiedBorder)}.inlineCompletions-char-delete.single-line-inline.empty,.inlineCompletions-char-insert.single-line-inline.empty{display:none}.inlineCompletions.strike-through{text-decoration-thickness:1px}.inlineCompletions-modified-bubble{background:var(--vscode-inlineEdit-modifiedChangedTextBackground)}.inlineCompletions-original-bubble{background:var(--vscode-inlineEdit-originalChangedTextBackground)}.inlineCompletions-modified-bubble,.inlineCompletions-original-bubble{pointer-events:none;display:inline-block}.inline-edit.ghost-text,.inline-edit.ghost-text-decoration,.inline-edit.ghost-text-decoration-preview,.inline-edit.suggest-preview-text .ghost-text{&.syntax-highlighted{opacity:1!important}font-style:normal!important}.inline-edit.modified-background.ghost-text,.inline-edit.modified-background.ghost-text-decoration,.inline-edit.modified-background.ghost-text-decoration-preview,.inline-edit.modified-background.suggest-preview-text .ghost-text{background:var(--vscode-inlineEdit-modifiedChangedTextBackground)!important;display:inline-block!important}.inlineCompletions-original-lines{background:var(--vscode-editor-background)}}.monaco-menu-option{color:var(--vscode-editorActionList-foreground);font-size:13px;padding:0 4px;line-height:28px;display:flex;gap:4px;align-items:center;border-radius:3px;cursor:pointer;.monaco-keybinding-key{font-size:13px;opacity:.7}&.active{background:var(--vscode-editorActionList-focusBackground);color:var(--vscode-editorActionList-focusForeground);outline:1px solid var(--vscode-menu-selectionBorder, transparent);outline-offset:-1px;.monaco-keybinding-key{color:var(--vscode-editorActionList-focusForeground)}}}.monaco-editor .goto-definition-link{text-decoration:underline;cursor:pointer;color:var(--vscode-editorLink-activeForeground)!important}.monaco-editor.vs .valueSetReplacement{outline:solid 2px var(--vscode-editorBracketMatch-border)}.monaco-editor .linked-editing-decoration{background-color:var(--vscode-editor-linkedEditingBackground);min-width:1px}.monaco-editor .detected-link,.monaco-editor .detected-link-active{text-decoration:underline;text-underline-position:under}.monaco-editor .detected-link-active{cursor:pointer;color:var(--vscode-editorLink-activeForeground)!important}.monaco-editor{.scroll-editor-on-middle-click-dot{cursor:all-scroll;position:absolute;z-index:1;background-color:var(--vscode-editor-foreground, white);border:1px solid var(--vscode-editor-background, black);opacity:.5;width:5px;height:5px;border-radius:50%;transform:translate(-50%,-50%);&.hidden{display:none}}&.scroll-editor-on-middle-click-editor *{cursor:all-scroll}}.monaco-editor .focused .selectionHighlight{background-color:var(--vscode-editor-selectionHighlightBackground);box-sizing:border-box;border:1px solid var(--vscode-editor-selectionHighlightBorder)}.monaco-editor.hc-black .focused .selectionHighlight,.monaco-editor.hc-light .focused .selectionHighlight{border-style:dotted}.monaco-editor .wordHighlight{background-color:var(--vscode-editor-wordHighlightBackground);box-sizing:border-box;border:1px solid var(--vscode-editor-wordHighlightBorder)}.monaco-editor.hc-black .wordHighlight,.monaco-editor.hc-light .wordHighlight{border-style:dotted}.monaco-editor .wordHighlightStrong{background-color:var(--vscode-editor-wordHighlightStrongBackground);box-sizing:border-box;border:1px solid var(--vscode-editor-wordHighlightStrongBorder)}.monaco-editor.hc-black .wordHighlightStrong,.monaco-editor.hc-light .wordHighlightStrong{border-style:dotted}.monaco-editor .wordHighlightText{background-color:var(--vscode-editor-wordHighlightTextBackground);box-sizing:border-box;border:1px solid var(--vscode-editor-wordHighlightTextBorder)}.monaco-editor.hc-black .wordHighlightText,.monaco-editor.hc-light .wordHighlightText{border-style:dotted}.monaco-editor .parameter-hints-widget{z-index:39;display:flex;flex-direction:column;line-height:1.5em;cursor:default;color:var(--vscode-editorHoverWidget-foreground);background-color:var(--vscode-editorHoverWidget-background);border:1px solid var(--vscode-editorHoverWidget-border)}.hc-black .monaco-editor .parameter-hints-widget,.hc-light .monaco-editor .parameter-hints-widget{border-width:2px}.monaco-editor .parameter-hints-widget>.phwrapper{max-width:440px;display:flex;flex-direction:row}.monaco-editor .parameter-hints-widget.multiple{min-height:3.3em;padding:0}.monaco-editor .parameter-hints-widget.multiple .body:before{content:\"\";display:block;height:100%;position:absolute;opacity:.5;border-left:1px solid var(--vscode-editorHoverWidget-border)}.monaco-editor .parameter-hints-widget p,.monaco-editor .parameter-hints-widget ul{margin:8px 0}.monaco-editor .parameter-hints-widget .monaco-scrollable-element,.monaco-editor .parameter-hints-widget .body{display:flex;flex:1;flex-direction:column;min-height:100%}.monaco-editor .parameter-hints-widget .signature{padding:4px 5px;position:relative}.monaco-editor .parameter-hints-widget .signature.has-docs:after{content:\"\";display:block;position:absolute;left:0;width:100%;padding-top:4px;opacity:.5;border-bottom:1px solid var(--vscode-editorHoverWidget-border)}.monaco-editor .parameter-hints-widget .code{font-family:var(--vscode-parameterHintsWidget-editorFontFamily),var(--vscode-parameterHintsWidget-editorFontFamilyDefault)}.monaco-editor .parameter-hints-widget .docs{padding:0 10px 0 5px;white-space:pre-wrap}.monaco-editor .parameter-hints-widget .docs.empty{display:none}.monaco-editor .parameter-hints-widget .docs a{color:var(--vscode-textLink-foreground)}.monaco-editor .parameter-hints-widget .docs a:hover{color:var(--vscode-textLink-activeForeground);cursor:pointer}.monaco-editor .parameter-hints-widget .docs .markdown-docs{white-space:initial}.monaco-editor .parameter-hints-widget .docs code{font-family:var(--monaco-monospace-font);border-radius:3px;padding:0 .4em;background-color:var(--vscode-textCodeBlock-background)}.monaco-editor .parameter-hints-widget .docs .monaco-tokenized-source,.monaco-editor .parameter-hints-widget .docs .code{white-space:pre-wrap}.monaco-editor .parameter-hints-widget .controls{display:none;flex-direction:column;align-items:center;min-width:22px;justify-content:flex-end}.monaco-editor .parameter-hints-widget.multiple .controls{display:flex;padding:0 2px}.monaco-editor .parameter-hints-widget.multiple .button{width:16px;height:16px;background-repeat:no-repeat;cursor:pointer}.monaco-editor .parameter-hints-widget .button.previous{bottom:24px}.monaco-editor .parameter-hints-widget .overloads{text-align:center;height:12px;line-height:12px;font-family:var(--monaco-monospace-font)}.monaco-editor .parameter-hints-widget .signature .parameter.active{color:var(--vscode-editorHoverWidget-highlightForeground);font-weight:700}.monaco-editor .parameter-hints-widget .documentation-parameter>.parameter{font-weight:700;margin-right:.5em}.monaco-editor{.editorPlaceholder{top:0;position:absolute;overflow:hidden;text-overflow:ellipsis;text-wrap:nowrap;pointer-events:none;color:var(--vscode-editor-placeholder-foreground)}}.monaco-editor .rename-box{z-index:100;color:inherit;border-radius:4px}.monaco-editor .rename-box.preview{padding:4px 4px 0}.monaco-editor .rename-box .rename-input-with-button{padding:3px;border-radius:2px;width:calc(100% - 8px)}.monaco-editor .rename-box .rename-input{width:calc(100% - 8px);padding:0}.monaco-editor .rename-box .rename-input:focus{outline:none}.monaco-editor .rename-box .rename-suggestions-button{display:flex;align-items:center;padding:3px;background-color:transparent;border:none;border-radius:5px;cursor:pointer}.monaco-editor .rename-box .rename-suggestions-button:hover{background-color:var(--vscode-toolbar-hoverBackground)}.monaco-editor .rename-box .rename-candidate-list-container .monaco-list-row{border-radius:2px}.monaco-editor .rename-box .rename-label{display:none;opacity:.8}.monaco-editor .rename-box.preview .rename-label{display:inherit}.monaco-editor .unicode-highlight{border:1px solid var(--vscode-editorUnicodeHighlight-border);background-color:var(--vscode-editorUnicodeHighlight-background);box-sizing:border-box}.editor-banner{box-sizing:border-box;cursor:default;width:100%;font-size:12px;display:flex;overflow:visible;height:26px;background:var(--vscode-banner-background)}.editor-banner .icon-container{display:flex;flex-shrink:0;align-items:center;padding:0 6px 0 10px}.editor-banner .icon-container.custom-icon{background-repeat:no-repeat;background-position:center center;background-size:16px;width:16px;padding:0;margin:0 6px 0 10px}.editor-banner .message-container{display:flex;align-items:center;line-height:26px;text-overflow:ellipsis;white-space:nowrap;overflow:hidden}.editor-banner .message-container p{margin-block-start:0;margin-block-end:0}.editor-banner .message-actions-container{flex-grow:1;flex-shrink:0;line-height:26px;margin:0 4px}.editor-banner .message-actions-container a.monaco-button{width:inherit;margin:2px 8px;padding:0 12px}.editor-banner .message-actions-container a{padding:3px;margin-left:12px;text-decoration:underline}.editor-banner .action-container{padding:0 10px 0 6px}.editor-banner{background-color:var(--vscode-banner-background)}.editor-banner,.editor-banner .action-container .codicon,.editor-banner .message-actions-container .monaco-link{color:var(--vscode-banner-foreground)}.editor-banner .icon-container .codicon{color:var(--vscode-banner-iconForeground)}.monaco-link{color:var(--vscode-textLink-foreground)}.monaco-link:hover{color:var(--vscode-textLink-activeForeground)}.floating-menu-overlay-widget{padding:0;color:var(--vscode-button-foreground);background-color:var(--vscode-button-background);border-radius:2px;border:1px solid var(--vscode-contrastBorder);display:flex;align-items:center;z-index:10;box-shadow:0 2px 8px var(--vscode-widget-shadow);overflow:hidden;.action-item>.action-label{padding:5px;font-size:12px;border-radius:2px}.action-item>.action-label.codicon{color:var(--vscode-button-foreground)}.action-item>.action-label.codicon:not(.separator){padding-top:6px;padding-bottom:6px}.action-item:first-child>.action-label{padding-left:7px}.action-item:last-child>.action-label{padding-right:7px}.action-item .action-label.separator{background-color:var(--vscode-menu-separatorBackground)}}.monaco-editor .iPadShowKeyboard{width:58px;min-width:0;height:36px;min-height:0;margin:0;padding:0;position:absolute;resize:none;overflow:hidden;background:url(data:image/svg+xml;base64,PHN2ZyB3aWR0aD0iNTMiIGhlaWdodD0iMzYiIHZpZXdCb3g9IjAgMCA1MyAzNiIgZmlsbD0ibm9uZSIgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIj4KPGcgY2xpcC1wYXRoPSJ1cmwoI2NsaXAwKSI+CjxwYXRoIGZpbGwtcnVsZT0iZXZlbm9kZCIgY2xpcC1ydWxlPSJldmVub2RkIiBkPSJNNDguMDM2NCA0LjAxMDQySDQuMDA3NzlMNC4wMDc3OSAzMi4wMjg2SDQ4LjAzNjRWNC4wMTA0MlpNNC4wMDc3OSAwLjAwNzgxMjVDMS43OTcyMSAwLjAwNzgxMjUgMC4wMDUxODc5OSAxLjc5OTg0IDAuMDA1MTg3OTkgNC4wMTA0MlYzMi4wMjg2QzAuMDA1MTg3OTkgMzQuMjM5MiAxLjc5NzIxIDM2LjAzMTIgNC4wMDc3OSAzNi4wMzEySDQ4LjAzNjRDNTAuMjQ3IDM2LjAzMTIgNTIuMDM5IDM0LjIzOTIgNTIuMDM5IDMyLjAyODZWNC4wMTA0MkM1Mi4wMzkgMS43OTk4NCA1MC4yNDcgMC4wMDc4MTI1IDQ4LjAzNjQgMC4wMDc4MTI1SDQuMDA3NzlaTTguMDEwNDIgOC4wMTMwMkgxMi4wMTNWMTIuMDE1Nkg4LjAxMDQyVjguMDEzMDJaTTIwLjAxODIgOC4wMTMwMkgxNi4wMTU2VjEyLjAxNTZIMjAuMDE4MlY4LjAxMzAyWk0yNC4wMjA4IDguMDEzMDJIMjguMDIzNFYxMi4wMTU2SDI0LjAyMDhWOC4wMTMwMlpNMzYuMDI4NiA4LjAxMzAySDMyLjAyNlYxMi4wMTU2SDM2LjAyODZWOC4wMTMwMlpNNDAuMDMxMiA4LjAxMzAySDQ0LjAzMzlWMTIuMDE1Nkg0MC4wMzEyVjguMDEzMDJaTTE2LjAxNTYgMTYuMDE4Mkg4LjAxMDQyVjIwLjAyMDhIMTYuMDE1NlYxNi4wMTgyWk0yMC4wMTgyIDE2LjAxODJIMjQuMDIwOFYyMC4wMjA4SDIwLjAxODJWMTYuMDE4MlpNMzIuMDI2IDE2LjAxODJIMjguMDIzNFYyMC4wMjA4SDMyLjAyNlYxNi4wMTgyWk00NC4wMzM5IDE2LjAxODJWMjAuMDIwOEgzNi4wMjg2VjE2LjAxODJINDQuMDMzOVpNMTIuMDEzIDI0LjAyMzRIOC4wMTA0MlYyOC4wMjZIMTIuMDEzVjI0LjAyMzRaTTE2LjAxNTYgMjQuMDIzNEgzNi4wMjg2VjI4LjAyNkgxNi4wMTU2VjI0LjAyMzRaTTQ0LjAzMzkgMjQuMDIzNEg0MC4wMzEyVjI4LjAyNkg0NC4wMzM5VjI0LjAyMzRaIiBmaWxsPSIjNDI0MjQyIi8+CjwvZz4KPGRlZnM+CjxjbGlwUGF0aCBpZD0iY2xpcDAiPgo8cmVjdCB3aWR0aD0iNTMiIGhlaWdodD0iMzYiIGZpbGw9IndoaXRlIi8+CjwvY2xpcFBhdGg+CjwvZGVmcz4KPC9zdmc+Cg==) center center no-repeat;border:4px solid #F6F6F6;border-radius:4px}.monaco-editor.vs-dark .iPadShowKeyboard{background:url(data:image/svg+xml;base64,PHN2ZyB3aWR0aD0iNTMiIGhlaWdodD0iMzYiIHZpZXdCb3g9IjAgMCA1MyAzNiIgZmlsbD0ibm9uZSIgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIj4KPGcgY2xpcC1wYXRoPSJ1cmwoI2NsaXAwKSI+CjxwYXRoIGZpbGwtcnVsZT0iZXZlbm9kZCIgY2xpcC1ydWxlPSJldmVub2RkIiBkPSJNNDguMDM2NCA0LjAxMDQySDQuMDA3NzlMNC4wMDc3OSAzMi4wMjg2SDQ4LjAzNjRWNC4wMTA0MlpNNC4wMDc3OSAwLjAwNzgxMjVDMS43OTcyMSAwLjAwNzgxMjUgMC4wMDUxODc5OSAxLjc5OTg0IDAuMDA1MTg3OTkgNC4wMTA0MlYzMi4wMjg2QzAuMDA1MTg3OTkgMzQuMjM5MiAxLjc5NzIxIDM2LjAzMTIgNC4wMDc3OSAzNi4wMzEySDQ4LjAzNjRDNTAuMjQ3IDM2LjAzMTIgNTIuMDM5IDM0LjIzOTIgNTIuMDM5IDMyLjAyODZWNC4wMTA0MkM1Mi4wMzkgMS43OTk4NCA1MC4yNDcgMC4wMDc4MTI1IDQ4LjAzNjQgMC4wMDc4MTI1SDQuMDA3NzlaTTguMDEwNDIgOC4wMTMwMkgxMi4wMTNWMTIuMDE1Nkg4LjAxMDQyVjguMDEzMDJaTTIwLjAxODIgOC4wMTMwMkgxNi4wMTU2VjEyLjAxNTZIMjAuMDE4MlY4LjAxMzAyWk0yNC4wMjA4IDguMDEzMDJIMjguMDIzNFYxMi4wMTU2SDI0LjAyMDhWOC4wMTMwMlpNMzYuMDI4NiA4LjAxMzAySDMyLjAyNlYxMi4wMTU2SDM2LjAyODZWOC4wMTMwMlpNNDAuMDMxMiA4LjAxMzAySDQ0LjAzMzlWMTIuMDE1Nkg0MC4wMzEyVjguMDEzMDJaTTE2LjAxNTYgMTYuMDE4Mkg4LjAxMDQyVjIwLjAyMDhIMTYuMDE1NlYxNi4wMTgyWk0yMC4wMTgyIDE2LjAxODJIMjQuMDIwOFYyMC4wMjA4SDIwLjAxODJWMTYuMDE4MlpNMzIuMDI2IDE2LjAxODJIMjguMDIzNFYyMC4wMjA4SDMyLjAyNlYxNi4wMTgyWk00NC4wMzM5IDE2LjAxODJWMjAuMDIwOEgzNi4wMjg2VjE2LjAxODJINDQuMDMzOVpNMTIuMDEzIDI0LjAyMzRIOC4wMTA0MlYyOC4wMjZIMTIuMDEzVjI0LjAyMzRaTTE2LjAxNTYgMjQuMDIzNEgzNi4wMjg2VjI4LjAyNkgxNi4wMTU2VjI0LjAyMzRaTTQ0LjAzMzkgMjQuMDIzNEg0MC4wMzEyVjI4LjAyNkg0NC4wMzM5VjI0LjAyMzRaIiBmaWxsPSIjQzVDNUM1Ii8+CjwvZz4KPGRlZnM+CjxjbGlwUGF0aCBpZD0iY2xpcDAiPgo8cmVjdCB3aWR0aD0iNTMiIGhlaWdodD0iMzYiIGZpbGw9IndoaXRlIi8+CjwvY2xpcFBhdGg+CjwvZGVmcz4KPC9zdmc+Cg==) center center no-repeat;border:4px solid #252526}.monaco-editor .tokens-inspect-widget{z-index:50;user-select:text;-webkit-user-select:text;padding:10px;color:var(--vscode-editorHoverWidget-foreground);background-color:var(--vscode-editorHoverWidget-background);border:1px solid var(--vscode-editorHoverWidget-border)}.monaco-editor.hc-black .tokens-inspect-widget,.monaco-editor.hc-light .tokens-inspect-widget{border-width:2px}.monaco-editor .tokens-inspect-widget .tokens-inspect-separator{height:1px;border:0;background-color:var(--vscode-editorHoverWidget-border)}.monaco-editor .tokens-inspect-widget .tm-token{font-family:var(--monaco-monospace-font)}.monaco-editor .tokens-inspect-widget .tm-token-length{font-weight:400;font-size:60%;float:right}.monaco-editor .tokens-inspect-widget .tm-metadata-table{width:100%}.monaco-editor .tokens-inspect-widget .tm-metadata-value{font-family:var(--monaco-monospace-font);text-align:right}.monaco-editor .tokens-inspect-widget .tm-token-type{font-family:var(--monaco-monospace-font)}.monaco-editor{font-family:-apple-system,BlinkMacSystemFont,Segoe WPC,Segoe UI,HelveticaNeue-Light,system-ui,Ubuntu,Droid Sans,sans-serif;--monaco-monospace-font: \"SF Mono\", Monaco, Menlo, Consolas, \"Ubuntu Mono\", \"Liberation Mono\", \"DejaVu Sans Mono\", \"Courier New\", monospace}.monaco-menu .monaco-action-bar.vertical .action-item .action-menu-item:focus .action-label{stroke-width:1.2px}.monaco-editor.vs-dark .monaco-menu .monaco-action-bar.vertical .action-menu-item:focus .action-label,.monaco-editor.hc-black .monaco-menu .monaco-action-bar.vertical .action-menu-item:focus .action-label,.monaco-editor.hc-light .monaco-menu .monaco-action-bar.vertical .action-menu-item:focus .action-label{stroke-width:1.2px}.monaco-hover p{margin:0}.monaco-aria-container{position:absolute!important;top:0;height:1px;width:1px;margin:-1px;overflow:hidden;padding:0;clip:rect(1px,1px,1px,1px);clip-path:inset(50%)}.monaco-editor .synthetic-focus,.monaco-diff-editor .synthetic-focus,.monaco-editor [tabindex=\"0\"]:focus,.monaco-diff-editor [tabindex=\"0\"]:focus,.monaco-editor [tabindex=\"-1\"]:focus,.monaco-diff-editor [tabindex=\"-1\"]:focus,.monaco-editor button:focus,.monaco-diff-editor button:focus,.monaco-editor input[type=button]:focus,.monaco-diff-editor input[type=button]:focus,.monaco-editor input[type=checkbox]:focus,.monaco-diff-editor input[type=checkbox]:focus,.monaco-editor input[type=search]:focus,.monaco-diff-editor input[type=search]:focus,.monaco-editor input[type=text]:focus,.monaco-diff-editor input[type=text]:focus,.monaco-editor select:focus,.monaco-diff-editor select:focus,.monaco-editor textarea:focus,.monaco-diff-editor textarea:focus{outline-width:1px;outline-style:solid;outline-offset:-1px;outline-color:var(--vscode-focusBorder);opacity:1}.monaco-hover.workbench-hover{position:relative;font-size:13px;line-height:19px;z-index:40;overflow:hidden;max-width:700px;background:var(--vscode-editorHoverWidget-background);border:1px solid var(--vscode-editorHoverWidget-border);border-radius:5px;color:var(--vscode-editorHoverWidget-foreground);box-shadow:0 2px 8px var(--vscode-widget-shadow)}.monaco-hover.workbench-hover .monaco-action-bar .action-item .codicon{width:13px;height:13px}.monaco-hover.workbench-hover hr{border-bottom:none}.monaco-hover.workbench-hover.compact{font-size:12px}.monaco-hover.workbench-hover.compact .monaco-action-bar .action-item .codicon{width:12px;height:12px}.monaco-hover.workbench-hover.compact .hover-contents{padding:2px 8px}.workbench-hover-container.locked .monaco-hover.workbench-hover{outline:1px solid var(--vscode-editorHoverWidget-border)}.workbench-hover-container:focus-within.locked .monaco-hover.workbench-hover{outline-color:var(--vscode-focusBorder)}.workbench-hover-pointer{position:absolute;z-index:41;pointer-events:none}.workbench-hover-pointer:after{content:\"\";position:absolute;width:5px;height:5px;background-color:var(--vscode-editorHoverWidget-background);border-right:1px solid var(--vscode-editorHoverWidget-border);border-bottom:1px solid var(--vscode-editorHoverWidget-border)}.workbench-hover-container:not(:focus-within).locked .workbench-hover-pointer:after{width:4px;height:4px;border-right-width:2px;border-bottom-width:2px}.workbench-hover-container:focus-within .workbench-hover-pointer:after{border-right:1px solid var(--vscode-focusBorder);border-bottom:1px solid var(--vscode-focusBorder)}.workbench-hover-pointer.left{left:-3px}.workbench-hover-pointer.right{right:3px}.workbench-hover-pointer.top{top:-3px}.workbench-hover-pointer.bottom{bottom:3px}.workbench-hover-pointer.left:after{transform:rotate(135deg)}.workbench-hover-pointer.right:after{transform:rotate(315deg)}.workbench-hover-pointer.top:after{transform:rotate(225deg)}.workbench-hover-pointer.bottom:after{transform:rotate(45deg)}.monaco-hover.workbench-hover a{color:var(--vscode-textLink-foreground)}.monaco-hover.workbench-hover a:focus{outline:1px solid;outline-offset:-1px;text-decoration:underline;outline-color:var(--vscode-focusBorder)}.monaco-hover.workbench-hover a.codicon:focus,.monaco-hover.workbench-hover a.monaco-button:focus{text-decoration:none}.monaco-hover.workbench-hover a:hover,.monaco-hover.workbench-hover a:active{color:var(--vscode-textLink-activeForeground)}.monaco-hover.workbench-hover code{background:var(--vscode-textCodeBlock-background)}.monaco-hover.workbench-hover .hover-row .actions{background:var(--vscode-editorHoverWidget-statusBarBackground)}.monaco-hover.workbench-hover.right-aligned{left:1px}.monaco-hover.workbench-hover.right-aligned .hover-row.status-bar .actions{flex-direction:row-reverse}.monaco-hover.workbench-hover.right-aligned .hover-row.status-bar .actions .action-container{margin-right:0;margin-left:16px}.context-view{position:absolute}.context-view.fixed{all:initial;font-family:inherit;font-size:13px;position:fixed;color:inherit}.quick-input-widget{font-size:13px}.quick-input-widget .monaco-highlighted-label .highlight{color:#0066bf}.vs .quick-input-widget .monaco-list-row.focused .monaco-highlighted-label .highlight{color:#9dddff}.vs-dark .quick-input-widget .monaco-highlighted-label .highlight{color:#0097fb}.hc-black .quick-input-widget .monaco-highlighted-label .highlight{color:#f38518}.hc-light .quick-input-widget .monaco-highlighted-label .highlight{color:#0f4a85}.monaco-keybinding>.monaco-keybinding-key{background-color:#ddd6;border:solid 1px rgba(204,204,204,.4);border-bottom-color:#bbb6;box-shadow:inset 0 -1px #bbb6;color:#555}.hc-black .monaco-keybinding>.monaco-keybinding-key{background-color:transparent;border:solid 1px rgb(111,195,223);box-shadow:none;color:#fff}.hc-light .monaco-keybinding>.monaco-keybinding-key{background-color:transparent;border:solid 1px #0F4A85;box-shadow:none;color:#292929}.vs-dark .monaco-keybinding>.monaco-keybinding-key{background-color:#8080802b;border:solid 1px rgba(51,51,51,.6);border-bottom-color:#4449;box-shadow:inset 0 -1px #4449;color:#ccc}.quick-input-widget{position:absolute;width:600px;z-index:2550;left:50%;-webkit-app-region:no-drag;border-radius:6px}.quick-input-titlebar{cursor:grab;display:flex;align-items:center;border-top-right-radius:5px;border-top-left-radius:5px}.quick-input-left-action-bar{display:flex;margin-left:4px;flex:1}.quick-input-inline-action-bar>.actions-container>.action-item:first-child{margin-left:5px}.quick-input-inline-action-bar>.actions-container>.action-item{margin-top:2px}.quick-input-title{cursor:grab;padding:3px 0;text-align:center;text-overflow:ellipsis;overflow:hidden}.quick-input-right-action-bar{display:flex;margin-right:4px;flex:1}.quick-input-right-action-bar>.actions-container{justify-content:flex-end}.quick-input-right-action-bar>.actions-container>.action-item{margin-left:4px}.quick-input-titlebar .monaco-action-bar .action-label.codicon{background-position:center;background-repeat:no-repeat;padding:2px}.quick-input-description{margin:6px 6px 6px 11px}.quick-input-header .quick-input-description{margin:4px 2px;flex:1}.quick-input-header{cursor:grab;display:flex;padding:6px 6px 2px}.quick-input-widget.hidden-input .quick-input-header{padding:0;margin-bottom:0}.quick-input-and-message{display:flex;flex-direction:column;flex-grow:1;min-width:0;position:relative}.quick-input-check-all{align-self:center;margin:0}.quick-input-widget .quick-input-header .monaco-checkbox{margin-top:6px}.quick-input-filter{flex-grow:1;display:flex;position:relative}.quick-input-box{flex-grow:1}.quick-input-widget.show-checkboxes .quick-input-box,.quick-input-widget.show-checkboxes .quick-input-message{margin-left:5px}.quick-input-visible-count{position:absolute;left:-10000px}.quick-input-count{align-self:center;position:absolute;right:4px;display:flex;align-items:center}.quick-input-count .monaco-count-badge{vertical-align:middle;padding:2px 4px;border-radius:2px;min-height:auto;line-height:normal}.quick-input-action{margin-left:6px}.quick-input-action .monaco-text-button{font-size:11px;padding:0 6px;display:flex;height:25px;align-items:center}.quick-input-message{margin-top:-1px;padding:5px;overflow-wrap:break-word}.quick-input-message>.codicon{margin:0 .2em;vertical-align:text-bottom}.quick-input-message a{color:inherit}.quick-input-progress.monaco-progress-container{position:relative}.quick-input-list{line-height:22px}.quick-input-widget.hidden-input .quick-input-list{margin-top:4px;padding-bottom:4px}.quick-input-list .monaco-list{overflow:hidden;max-height:440px;padding-bottom:5px}.quick-input-list .monaco-scrollable-element{padding:0 6px}.quick-input-list .quick-input-list-entry{box-sizing:border-box;overflow:hidden;display:flex;padding:0 6px}.quick-input-list .quick-input-list-entry.quick-input-list-separator-border{border-top-width:1px;border-top-style:solid}.quick-input-list .monaco-list-row{border-radius:3px}.quick-input-list .monaco-list-row[data-index=\"0\"] .quick-input-list-entry.quick-input-list-separator-border{border-top-style:none}.quick-input-list .quick-input-list-label{overflow:hidden;display:flex;height:100%;flex:1}.quick-input-widget .monaco-checkbox{margin-right:0}.quick-input-widget .quick-input-list .monaco-checkbox,.quick-input-widget .quick-input-tree .monaco-checkbox{margin-top:4px}.quick-input-list .quick-input-list-icon{background-size:16px;background-position:left center;background-repeat:no-repeat;padding-right:6px;width:16px;height:22px;display:flex;align-items:center;justify-content:center}.quick-input-list .quick-input-list-rows{overflow:hidden;text-overflow:ellipsis;display:flex;flex-direction:column;height:100%;flex:1;margin-left:5px}.quick-input-list .quick-input-list-rows>.quick-input-list-row{display:flex;align-items:center}.quick-input-list .quick-input-list-rows>.quick-input-list-row .monaco-icon-label,.quick-input-list .quick-input-list-rows>.quick-input-list-row .monaco-icon-label .monaco-icon-label-container>.monaco-icon-name-container{flex:1}.quick-input-list .quick-input-list-rows>.quick-input-list-row .codicon[class*=codicon-]{vertical-align:text-bottom}.quick-input-list .quick-input-list-rows .monaco-highlighted-label>span{opacity:1}.quick-input-list .quick-input-list-entry .quick-input-list-entry-keybinding{margin-right:8px}.quick-input-list .quick-input-list-label-meta{opacity:.7;line-height:normal;text-overflow:ellipsis;overflow:hidden}.quick-input-list .monaco-list .monaco-list-row .monaco-highlighted-label .highlight{font-weight:700;background-color:unset;color:var(--vscode-list-highlightForeground)!important}.quick-input-list .monaco-list .monaco-list-row.focused .monaco-highlighted-label .highlight{color:var(--vscode-list-focusHighlightForeground)!important}.quick-input-list .quick-input-list-entry .quick-input-list-separator{margin-right:4px}.quick-input-list .quick-input-list-entry-action-bar{display:flex;flex:0;overflow:visible}.quick-input-list .quick-input-list-entry-action-bar .action-label{display:none}.quick-input-list .quick-input-list-entry-action-bar .action-label.codicon{margin-right:4px;padding:2px}.quick-input-list .quick-input-list-entry-action-bar{margin-top:1px}.quick-input-list .quick-input-list-entry-action-bar{margin-right:4px}.quick-input-list .quick-input-list-entry .quick-input-list-entry-action-bar .action-label.always-visible,.quick-input-list .quick-input-list-entry:hover .quick-input-list-entry-action-bar .action-label,.quick-input-list .quick-input-list-entry.focus-inside .quick-input-list-entry-action-bar .action-label,.quick-input-list .monaco-list-row.focused .quick-input-list-entry-action-bar .action-label,.quick-input-list .monaco-list-row.passive-focused .quick-input-list-entry-action-bar .action-label{display:flex}.quick-input-list>.monaco-list:focus .monaco-list-row.focused{outline:1px solid var(--vscode-list-focusOutline)!important;outline-offset:-1px}.quick-input-list>.monaco-list:focus .monaco-list-row.focused .quick-input-list-entry.quick-input-list-separator-border{border-color:transparent}.quick-input-list .monaco-list-row.focused .monaco-keybinding-key,.quick-input-list .monaco-list-row.focused .quick-input-list-entry .quick-input-list-separator{color:inherit}.quick-input-list .monaco-list-row.focused .monaco-keybinding-key{background:none}.quick-input-list .quick-input-list-separator-as-item{padding:4px 6px;font-size:12px}.quick-input-list .quick-input-list-separator-as-item .label-name{font-weight:600}.quick-input-list .quick-input-list-separator-as-item .label-description{opacity:1!important}.quick-input-list .monaco-tree-sticky-row .quick-input-list-entry.quick-input-list-separator-as-item.quick-input-list-separator-border{border-top-style:none}.quick-input-list .monaco-tree-sticky-row{padding:0 5px}.quick-input-list .monaco-tl-twistie{display:none!important}.quick-input-tree .monaco-list{overflow:hidden;max-height:440px;padding-bottom:5px}.quick-input-tree .quick-input-tree-entry{box-sizing:border-box;overflow:hidden;display:flex;padding:0 6px}.quick-input-tree .quick-input-tree-label{overflow:hidden;display:flex;height:100%;flex:1}.quick-input-tree .quick-input-tree-icon{background-size:16px;background-position:left center;background-repeat:no-repeat;padding-right:6px;width:16px;height:22px;display:flex;align-items:center;justify-content:center}.quick-input-tree .quick-input-tree-rows{overflow:hidden;text-overflow:ellipsis;display:flex;flex-direction:column;height:100%;flex:1;margin-left:5px}.quick-input-tree .quick-input-tree-rows>.quick-input-tree-row{display:flex;align-items:center}.quick-input-tree .quick-input-tree-rows>.quick-input-tree-row .monaco-icon-label,.quick-input-tree .quick-input-tree-rows>.quick-input-tree-row .monaco-icon-label .monaco-icon-label-container>.monaco-icon-name-container{flex:1}.quick-input-tree .quick-input-tree-rows>.quick-input-tree-row .codicon[class*=codicon-]{vertical-align:text-bottom}.quick-input-tree .quick-input-tree-rows .monaco-highlighted-label>span{opacity:1}.quick-input-tree .quick-input-tree-entry-action-bar{display:flex;flex:0;overflow:visible}.quick-input-tree .quick-input-tree-entry-action-bar .action-label{display:none}.quick-input-tree .quick-input-tree-entry-action-bar .action-label.codicon{margin-right:4px;padding:2px}.quick-input-tree .quick-input-tree-entry-action-bar{margin-top:1px}.quick-input-tree .quick-input-tree-entry-action-bar{margin-right:4px}.quick-input-tree .quick-input-tree-entry .quick-input-tree-entry-action-bar .action-label.always-visible,.quick-input-tree .quick-input-tree-entry:hover .quick-input-tree-entry-action-bar .action-label,.quick-input-tree .quick-input-tree-entry.focus-inside .quick-input-tree-entry-action-bar .action-label,.quick-input-tree .monaco-list-row.focused .quick-input-tree-entry-action-bar .action-label,.quick-input-tree .monaco-list-row.passive-focused .quick-input-tree-entry-action-bar .action-label{display:flex}.quick-input-tree>.monaco-list:focus .monaco-list-row.focused{outline:1px solid var(--vscode-list-focusOutline)!important;outline-offset:-1px}.monaco-progress-container{width:100%;height:2px;overflow:hidden}.monaco-progress-container .progress-bit{width:2%;height:2px;position:absolute;left:0;display:none}.monaco-progress-container.active .progress-bit{display:inherit}.monaco-progress-container.discrete .progress-bit{left:0;transition:width .1s linear}.monaco-progress-container.discrete.done .progress-bit{width:100%}.monaco-progress-container.infinite .progress-bit{animation-name:progress;animation-duration:4s;animation-iteration-count:infinite;transform:translateZ(0);animation-timing-function:linear}.monaco-progress-container.infinite.infinite-long-running .progress-bit{animation-timing-function:steps(100)}@keyframes progress{0%{transform:translate(0) scaleX(1)}50%{transform:translate(2500%) scaleX(3)}to{transform:translate(4900%) scaleX(1)}}.monaco-editor .rendered-markdown kbd{background-color:var(--vscode-keybindingLabel-background);color:var(--vscode-keybindingLabel-foreground);border-style:solid;border-width:1px;border-radius:3px;border-color:var(--vscode-keybindingLabel-border);border-bottom-color:var(--vscode-keybindingLabel-bottomBorder);box-shadow:inset 0 -1px 0 var(--vscode-widget-shadow);vertical-align:middle;padding:1px 3px}.rendered-markdown li:has(input[type=checkbox]){list-style-type:none}.monaco-component.multiDiffEditor{background:var(--vscode-multiDiffEditor-background);position:relative;height:100%;width:100%;overflow-y:hidden;>div{position:absolute;top:0;left:0;height:100%;width:100%;&.placeholder{visibility:hidden;&.visible{visibility:visible}display:grid;place-items:center;place-content:center}}.active{--vscode-multiDiffEditor-border: var(--vscode-focusBorder)}.multiDiffEntry{display:flex;flex-direction:column;flex:1;overflow:hidden;.collapse-button{margin:0 5px;cursor:pointer;a{display:block}}.header{z-index:1000;background:var(--vscode-editor-background);&:not(.collapsed) .header-content{border-bottom:1px solid var(--vscode-sideBarSectionHeader-border)}.header-content{margin:8px 0 0;padding:4px 5px;border-top:1px solid var(--vscode-multiDiffEditor-border);display:flex;align-items:center;color:var(--vscode-foreground);background:var(--vscode-multiDiffEditor-headerBackground);&.shadow{box-shadow:var(--vscode-scrollbar-shadow) 0 6px 6px -6px}.file-path{display:flex;flex:1;min-width:0;.title{font-size:14px;line-height:22px;&.original{flex:1;min-width:0;text-overflow:ellipsis}}.status{font-weight:600;opacity:.75;margin:0 10px;line-height:22px}}.actions{padding:0 8px}}}.editorParent{flex:1;display:flex;flex-direction:column;border-bottom:1px solid var(--vscode-multiDiffEditor-border);overflow:hidden}.editorContainer{flex:1}}}\n";if(typeof document!=="undefined"&&!document.getElementById("tytus-workbench-css")){const s=document.createElement("style");s.id="tytus-workbench-css";s.textContent=__tytusWorkbenchCss;document.head.appendChild(s);}
import { jsx as a, jsxs as u, Fragment as De } from "react/jsx-runtime";
import { forwardRef as Ea, createElement as Vn, useState as T, useRef as Se, useCallback as N, useEffect as oe, createContext as gs, useMemo as ue, useContext as ws, lazy as ks, Suspense as xs } from "react";
import { createPortal as Ps } from "react-dom";
/**
 * @license lucide-react v0.562.0 - ISC
 *
 * This source code is licensed under the ISC license.
 * See the LICENSE file in the root directory of this source tree.
 */
const Ns = (e) => e.replace(/([a-z0-9])([A-Z])/g, "$1-$2").toLowerCase(), Ts = (e) => e.replace(
  /^([A-Z])|[\s-_]+(\w)/g,
  (t, n, r) => r ? r.toUpperCase() : n.toLowerCase()
), wa = (e) => {
  const t = Ts(e);
  return t.charAt(0).toUpperCase() + t.slice(1);
}, qa = (...e) => e.filter((t, n, r) => !!t && t.trim() !== "" && r.indexOf(t) === n).join(" ").trim(), zs = (e) => {
  for (const t in e)
    if (t.startsWith("aria-") || t === "role" || t === "title")
      return !0;
};
/**
 * @license lucide-react v0.562.0 - ISC
 *
 * This source code is licensed under the ISC license.
 * See the LICENSE file in the root directory of this source tree.
 */
var As = {
  xmlns: "http://www.w3.org/2000/svg",
  width: 24,
  height: 24,
  viewBox: "0 0 24 24",
  fill: "none",
  stroke: "currentColor",
  strokeWidth: 2,
  strokeLinecap: "round",
  strokeLinejoin: "round"
};
/**
 * @license lucide-react v0.562.0 - ISC
 *
 * This source code is licensed under the ISC license.
 * See the LICENSE file in the root directory of this source tree.
 */
const Ls = Ea(
  ({
    color: e = "currentColor",
    size: t = 24,
    strokeWidth: n = 2,
    absoluteStrokeWidth: r,
    className: i = "",
    children: s,
    iconNode: o,
    ...c
  }, m) => Vn(
    "svg",
    {
      ref: m,
      ...As,
      width: t,
      height: t,
      stroke: e,
      strokeWidth: r ? Number(n) * 24 / Number(t) : n,
      className: qa("lucide", i),
      ...!s && !zs(c) && { "aria-hidden": "true" },
      ...c
    },
    [
      ...o.map(([b, p]) => Vn(b, p)),
      ...Array.isArray(s) ? s : [s]
    ]
  )
);
/**
 * @license lucide-react v0.562.0 - ISC
 *
 * This source code is licensed under the ISC license.
 * See the LICENSE file in the root directory of this source tree.
 */
const de = (e, t) => {
  const n = Ea(
    ({ className: r, ...i }, s) => Vn(Ls, {
      ref: s,
      iconNode: t,
      className: qa(
        `lucide-${Ns(wa(e))}`,
        `lucide-${e}`,
        r
      ),
      ...i
    })
  );
  return n.displayName = wa(e), n;
};
/**
 * @license lucide-react v0.562.0 - ISC
 *
 * This source code is licensed under the ISC license.
 * See the LICENSE file in the root directory of this source tree.
 */
const Xs = [
  ["path", { d: "m5 12 7-7 7 7", key: "hav0vg" }],
  ["path", { d: "M12 19V5", key: "x0mq9r" }]
], Cs = de("arrow-up", Xs);
/**
 * @license lucide-react v0.562.0 - ISC
 *
 * This source code is licensed under the ISC license.
 * See the LICENSE file in the root directory of this source tree.
 */
const Os = [
  ["path", { d: "M12 8V4H8", key: "hb8ula" }],
  ["rect", { width: "16", height: "12", x: "4", y: "8", rx: "2", key: "enze0r" }],
  ["path", { d: "M2 14h2", key: "vft8re" }],
  ["path", { d: "M20 14h2", key: "4cs60a" }],
  ["path", { d: "M15 13v2", key: "1xurst" }],
  ["path", { d: "M9 13v2", key: "rq6x2g" }]
], Un = de("bot", Os);
/**
 * @license lucide-react v0.562.0 - ISC
 *
 * This source code is licensed under the ISC license.
 * See the LICENSE file in the root directory of this source tree.
 */
const Ss = [
  ["path", { d: "M12 20v-9", key: "1qisl0" }],
  ["path", { d: "M14 7a4 4 0 0 1 4 4v3a6 6 0 0 1-12 0v-3a4 4 0 0 1 4-4z", key: "uouzyp" }],
  ["path", { d: "M14.12 3.88 16 2", key: "qol33r" }],
  ["path", { d: "M21 21a4 4 0 0 0-3.81-4", key: "1b0z45" }],
  ["path", { d: "M21 5a4 4 0 0 1-3.55 3.97", key: "5cxbf6" }],
  ["path", { d: "M22 13h-4", key: "1jl80f" }],
  ["path", { d: "M3 21a4 4 0 0 1 3.81-4", key: "1fjd4g" }],
  ["path", { d: "M3 5a4 4 0 0 0 3.55 3.97", key: "1d7oge" }],
  ["path", { d: "M6 13H2", key: "82j7cp" }],
  ["path", { d: "m8 2 1.88 1.88", key: "fmnt4t" }],
  ["path", { d: "M9 7.13V6a3 3 0 1 1 6 0v1.13", key: "1vgav8" }]
], Hs = de("bug", Ss);
/**
 * @license lucide-react v0.562.0 - ISC
 *
 * This source code is licensed under the ISC license.
 * See the LICENSE file in the root directory of this source tree.
 */
const js = [["path", { d: "M20 6 9 17l-5-5", key: "1gmf2c" }]], Ms = de("check", js);
/**
 * @license lucide-react v0.562.0 - ISC
 *
 * This source code is licensed under the ISC license.
 * See the LICENSE file in the root directory of this source tree.
 */
const Ws = [["path", { d: "m6 9 6 6 6-6", key: "qrunsl" }]], Xt = de("chevron-down", Ws);
/**
 * @license lucide-react v0.562.0 - ISC
 *
 * This source code is licensed under the ISC license.
 * See the LICENSE file in the root directory of this source tree.
 */
const Vs = [
  ["path", { d: "M12 6v6l4 2", key: "mmk7yg" }],
  ["circle", { cx: "12", cy: "12", r: "10", key: "1mglay" }]
], Ya = de("clock", Vs);
/**
 * @license lucide-react v0.562.0 - ISC
 *
 * This source code is licensed under the ISC license.
 * See the LICENSE file in the root directory of this source tree.
 */
const Is = [
  ["rect", { width: "14", height: "14", x: "8", y: "8", rx: "2", ry: "2", key: "17jyea" }],
  ["path", { d: "M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2", key: "zix9uf" }]
], In = de("copy", Is);
/**
 * @license lucide-react v0.562.0 - ISC
 *
 * This source code is licensed under the ISC license.
 * See the LICENSE file in the root directory of this source tree.
 */
const Fs = [
  ["circle", { cx: "12", cy: "12", r: "1", key: "41hilf" }],
  ["circle", { cx: "19", cy: "12", r: "1", key: "1wjl8i" }],
  ["circle", { cx: "5", cy: "12", r: "1", key: "1pcz8c" }]
], Ja = de("ellipsis", Fs);
/**
 * @license lucide-react v0.562.0 - ISC
 *
 * This source code is licensed under the ISC license.
 * See the LICENSE file in the root directory of this source tree.
 */
const Ds = [
  [
    "path",
    {
      d: "M2.062 12.348a1 1 0 0 1 0-.696 10.75 10.75 0 0 1 19.876 0 1 1 0 0 1 0 .696 10.75 10.75 0 0 1-19.876 0",
      key: "1nclc0"
    }
  ],
  ["circle", { cx: "12", cy: "12", r: "3", key: "1v7zrd" }]
], Bn = de("eye", Ds);
/**
 * @license lucide-react v0.562.0 - ISC
 *
 * This source code is licensed under the ISC license.
 * See the LICENSE file in the root directory of this source tree.
 */
const Rs = [
  [
    "path",
    {
      d: "M4 12.15V4a2 2 0 0 1 2-2h8a2.4 2.4 0 0 1 1.706.706l3.588 3.588A2.4 2.4 0 0 1 20 8v12a2 2 0 0 1-2 2h-3.35",
      key: "1wthlu"
    }
  ],
  ["path", { d: "M14 2v5a1 1 0 0 0 1 1h5", key: "wfsgrz" }],
  ["path", { d: "m5 16-3 3 3 3", key: "331omg" }],
  ["path", { d: "m9 22 3-3-3-3", key: "lsp7cz" }]
], Gn = de("file-code-corner", Rs);
/**
 * @license lucide-react v0.562.0 - ISC
 *
 * This source code is licensed under the ISC license.
 * See the LICENSE file in the root directory of this source tree.
 */
const Es = [
  [
    "path",
    {
      d: "M11.35 22H6a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h8a2.4 2.4 0 0 1 1.706.706l3.588 3.588A2.4 2.4 0 0 1 20 8v5.35",
      key: "17jvcc"
    }
  ],
  ["path", { d: "M14 2v5a1 1 0 0 0 1 1h5", key: "wfsgrz" }],
  ["path", { d: "M14 19h6", key: "bvotb8" }],
  ["path", { d: "M17 16v6", key: "18yu1i" }]
], Qn = de("file-plus-corner", Es);
/**
 * @license lucide-react v0.562.0 - ISC
 *
 * This source code is licensed under the ISC license.
 * See the LICENSE file in the root directory of this source tree.
 */
const qs = [
  [
    "path",
    {
      d: "M6 22a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h8a2.4 2.4 0 0 1 1.704.706l3.588 3.588A2.4 2.4 0 0 1 20 8v12a2 2 0 0 1-2 2z",
      key: "1oefj6"
    }
  ],
  ["path", { d: "M14 2v5a1 1 0 0 0 1 1h5", key: "wfsgrz" }],
  ["circle", { cx: "11.5", cy: "14.5", r: "2.5", key: "1bq0ko" }],
  ["path", { d: "M13.3 16.3 15 18", key: "2quom7" }]
], on = de("file-search", qs);
/**
 * @license lucide-react v0.562.0 - ISC
 *
 * This source code is licensed under the ISC license.
 * See the LICENSE file in the root directory of this source tree.
 */
const Ys = [
  [
    "path",
    {
      d: "M6 22a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h8a2.4 2.4 0 0 1 1.704.706l3.588 3.588A2.4 2.4 0 0 1 20 8v12a2 2 0 0 1-2 2z",
      key: "1oefj6"
    }
  ],
  ["path", { d: "M14 2v5a1 1 0 0 0 1 1h5", key: "wfsgrz" }]
], Za = de("file", Ys);
/**
 * @license lucide-react v0.562.0 - ISC
 *
 * This source code is licensed under the ISC license.
 * See the LICENSE file in the root directory of this source tree.
 */
const Js = [
  [
    "path",
    {
      d: "m6 14 1.5-2.9A2 2 0 0 1 9.24 10H20a2 2 0 0 1 1.94 2.5l-1.54 6a2 2 0 0 1-1.95 1.5H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h3.9a2 2 0 0 1 1.69.9l.81 1.2a2 2 0 0 0 1.67.9H18a2 2 0 0 1 2 2v2",
      key: "usdka0"
    }
  ]
], $n = de("folder-open", Js);
/**
 * @license lucide-react v0.562.0 - ISC
 *
 * This source code is licensed under the ISC license.
 * See the LICENSE file in the root directory of this source tree.
 */
const Zs = [
  [
    "path",
    {
      d: "M20 20a2 2 0 0 0 2-2V8a2 2 0 0 0-2-2h-7.9a2 2 0 0 1-1.69-.9L9.6 3.9A2 2 0 0 0 7.93 3H4a2 2 0 0 0-2 2v13a2 2 0 0 0 2 2Z",
      key: "1kt360"
    }
  ]
], Ka = de("folder", Zs);
/**
 * @license lucide-react v0.562.0 - ISC
 *
 * This source code is licensed under the ISC license.
 * See the LICENSE file in the root directory of this source tree.
 */
const Ks = [
  ["line", { x1: "6", x2: "6", y1: "3", y2: "15", key: "17qcm7" }],
  ["circle", { cx: "18", cy: "6", r: "3", key: "1h7g24" }],
  ["circle", { cx: "6", cy: "18", r: "3", key: "fqmcym" }],
  ["path", { d: "M18 9a9 9 0 0 1-9 9", key: "n2h4wq" }]
], Us = de("git-branch", Ks);
/**
 * @license lucide-react v0.562.0 - ISC
 *
 * This source code is licensed under the ISC license.
 * See the LICENSE file in the root directory of this source tree.
 */
const Bs = [
  [
    "path",
    {
      d: "M22 17a2 2 0 0 1-2 2H6.828a2 2 0 0 0-1.414.586l-2.202 2.202A.71.71 0 0 1 2 21.286V5a2 2 0 0 1 2-2h16a2 2 0 0 1 2 2z",
      key: "18887p"
    }
  ],
  ["path", { d: "M7 11h10", key: "1twpyw" }],
  ["path", { d: "M7 15h6", key: "d9of3u" }],
  ["path", { d: "M7 7h8", key: "af5zfr" }]
], Gs = de("message-square-text", Bs);
/**
 * @license lucide-react v0.562.0 - ISC
 *
 * This source code is licensed under the ISC license.
 * See the LICENSE file in the root directory of this source tree.
 */
const Qs = [
  ["path", { d: "M12 19v3", key: "npa21l" }],
  ["path", { d: "M19 10v2a7 7 0 0 1-14 0v-2", key: "1vc78b" }],
  ["rect", { x: "9", y: "2", width: "6", height: "13", rx: "3", key: "s6n7sd" }]
], $s = de("mic", Qs);
/**
 * @license lucide-react v0.562.0 - ISC
 *
 * This source code is licensed under the ISC license.
 * See the LICENSE file in the root directory of this source tree.
 */
const _s = [
  ["rect", { width: "18", height: "18", x: "3", y: "3", rx: "2", key: "afitv7" }],
  ["path", { d: "M15 3v18", key: "14nvp0" }]
], ei = de("panel-right", _s);
/**
 * @license lucide-react v0.562.0 - ISC
 *
 * This source code is licensed under the ISC license.
 * See the LICENSE file in the root directory of this source tree.
 */
const ti = [
  [
    "path",
    {
      d: "m16 6-8.414 8.586a2 2 0 0 0 2.829 2.829l8.414-8.586a4 4 0 1 0-5.657-5.657l-8.379 8.551a6 6 0 1 0 8.485 8.485l8.379-8.551",
      key: "1miecu"
    }
  ]
], ka = de("paperclip", ti);
/**
 * @license lucide-react v0.562.0 - ISC
 *
 * This source code is licensed under the ISC license.
 * See the LICENSE file in the root directory of this source tree.
 */
const ni = [
  ["path", { d: "M5 12h14", key: "1ays0h" }],
  ["path", { d: "M12 5v14", key: "s699le" }]
], Fn = de("plus", ni);
/**
 * @license lucide-react v0.562.0 - ISC
 *
 * This source code is licensed under the ISC license.
 * See the LICENSE file in the root directory of this source tree.
 */
const ai = [
  ["path", { d: "M21 12a9 9 0 0 0-9-9 9.75 9.75 0 0 0-6.74 2.74L3 8", key: "14sxne" }],
  ["path", { d: "M3 3v5h5", key: "1xhq8a" }],
  ["path", { d: "M3 12a9 9 0 0 0 9 9 9.75 9.75 0 0 0 6.74-2.74L21 16", key: "1hlbsb" }],
  ["path", { d: "M16 16h5v5", key: "ccwih5" }]
], bt = de("refresh-ccw", ai);
/**
 * @license lucide-react v0.562.0 - ISC
 *
 * This source code is licensed under the ISC license.
 * See the LICENSE file in the root directory of this source tree.
 */
const ri = [
  ["path", { d: "m21 21-4.34-4.34", key: "14j7rj" }],
  ["circle", { cx: "11", cy: "11", r: "8", key: "4ej97u" }]
], Ua = de("search", ri);
/**
 * @license lucide-react v0.562.0 - ISC
 *
 * This source code is licensed under the ISC license.
 * See the LICENSE file in the root directory of this source tree.
 */
const si = [
  [
    "path",
    {
      d: "M9.671 4.136a2.34 2.34 0 0 1 4.659 0 2.34 2.34 0 0 0 3.319 1.915 2.34 2.34 0 0 1 2.33 4.033 2.34 2.34 0 0 0 0 3.831 2.34 2.34 0 0 1-2.33 4.033 2.34 2.34 0 0 0-3.319 1.915 2.34 2.34 0 0 1-4.659 0 2.34 2.34 0 0 0-3.32-1.915 2.34 2.34 0 0 1-2.33-4.033 2.34 2.34 0 0 0 0-3.831A2.34 2.34 0 0 1 6.35 6.051a2.34 2.34 0 0 0 3.319-1.915",
      key: "1i5ecw"
    }
  ],
  ["circle", { cx: "12", cy: "12", r: "3", key: "1v7zrd" }]
], ii = de("settings", si);
/**
 * @license lucide-react v0.562.0 - ISC
 *
 * This source code is licensed under the ISC license.
 * See the LICENSE file in the root directory of this source tree.
 */
const oi = [
  ["path", { d: "M10 5H3", key: "1qgfaw" }],
  ["path", { d: "M12 19H3", key: "yhmn1j" }],
  ["path", { d: "M14 3v4", key: "1sua03" }],
  ["path", { d: "M16 17v4", key: "1q0r14" }],
  ["path", { d: "M21 12h-9", key: "1o4lsq" }],
  ["path", { d: "M21 19h-5", key: "1rlt1p" }],
  ["path", { d: "M21 5h-7", key: "1oszz2" }],
  ["path", { d: "M8 10v4", key: "tgpxqk" }],
  ["path", { d: "M8 12H3", key: "a7s4jb" }]
], Ba = de("sliders-horizontal", oi);
/**
 * @license lucide-react v0.562.0 - ISC
 *
 * This source code is licensed under the ISC license.
 * See the LICENSE file in the root directory of this source tree.
 */
const ci = [
  ["rect", { width: "18", height: "18", x: "3", y: "3", rx: "2", key: "afitv7" }]
], Ga = de("square", ci);
/**
 * @license lucide-react v0.562.0 - ISC
 *
 * This source code is licensed under the ISC license.
 * See the LICENSE file in the root directory of this source tree.
 */
const li = [
  ["path", { d: "M10 11v6", key: "nco0om" }],
  ["path", { d: "M14 11v6", key: "outv1u" }],
  ["path", { d: "M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6", key: "miytrc" }],
  ["path", { d: "M3 6h18", key: "d0wm0j" }],
  ["path", { d: "M8 6V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2", key: "e791ji" }]
], xa = de("trash-2", li);
/**
 * @license lucide-react v0.562.0 - ISC
 *
 * This source code is licensed under the ISC license.
 * See the LICENSE file in the root directory of this source tree.
 */
const di = [
  ["path", { d: "M18 6 6 18", key: "1bl5f8" }],
  ["path", { d: "m6 6 12 12", key: "d8bk6v" }]
], qe = de("x", di), ui = {
  md: "markdown",
  markdown: "markdown",
  json: "json",
  jsonc: "json",
  ts: "typescript",
  tsx: "typescript",
  js: "javascript",
  jsx: "javascript",
  mjs: "javascript",
  cjs: "javascript",
  css: "css",
  scss: "css",
  less: "css",
  html: "html",
  htm: "html",
  xml: "xml",
  svg: "xml",
  yml: "yaml",
  yaml: "yaml",
  py: "python",
  sh: "shell",
  bash: "shell",
  zsh: "shell",
  csv: "csv",
  txt: "text",
  log: "text"
};
function Qa(e) {
  const t = e.split(".").pop()?.toLowerCase() ?? "";
  return ui[t] ?? "text";
}
function $a(e) {
  return {
    markdown: "Markdown",
    json: "JSON",
    typescript: "TypeScript",
    javascript: "JavaScript",
    css: "CSS",
    html: "HTML",
    xml: "XML",
    yaml: "YAML",
    python: "Python",
    shell: "Shell",
    csv: "CSV",
    text: "Plain Text"
  }[e];
}
function _a(e, t) {
  if (t > 15e5) return !1;
  const n = e.split(".").pop()?.toLowerCase() ?? "";
  return !(/* @__PURE__ */ new Set(["png", "jpg", "jpeg", "gif", "webp", "avif", "mp3", "wav", "flac", "mp4", "mov", "zip", "gz", "tar", "pdf", "dmg", "sqlite", "db"])).has(n);
}
const hi = /* @__PURE__ */ new Set([".git", "node_modules", "dist", "build", ".next", ".turbo", "coverage", ".cache"]);
function mi() {
  const e = _n();
  return !!(e?.showOpenFilePicker && e.showDirectoryPicker);
}
async function fi() {
  const e = _n()?.showOpenFilePicker;
  if (!e) return er(!1);
  const t = await e({ multiple: !0 }), n = await Promise.all(t.map((r) => ea(r, r.name, "local-file")));
  return St(n.filter(Boolean));
}
async function pi() {
  const e = _n()?.showDirectoryPicker;
  if (!e)
    return { name: "Browser fallback folder", files: await er(!0) };
  const t = await e({ mode: "readwrite" }), n = [];
  return await ta(t, t.name, n, 0, 320), { name: t.name, handle: t, files: St(n) };
}
async function bi(e) {
  const t = await Promise.all(e.map((n) => ea(n, n.name, "local-file")));
  return St(t.filter(Boolean));
}
async function Pa(e) {
  const t = [];
  return await ta(e, e.name, t, 0, 320), { name: e.name, handle: e, files: St(t) };
}
async function zn(e, t = "readwrite") {
  try {
    return await e.queryPermission?.({ mode: t }) === "granted" ? !0 : await e.requestPermission?.({ mode: t }) === "granted" || !e.queryPermission && !e.requestPermission;
  } catch {
    return !1;
  }
}
function _n() {
  const e = window;
  if (typeof e.showOpenFilePicker == "function" && typeof e.showDirectoryPicker == "function")
    return e;
  try {
    const t = window.top;
    if (t && t !== e && typeof t.showOpenFilePicker == "function" && typeof t.showDirectoryPicker == "function")
      return t;
  } catch {
  }
  return null;
}
async function ea(e, t, n) {
  const r = await e.getFile();
  if (!_a(t, r.size)) return null;
  const i = await r.text();
  return {
    id: tr(t),
    name: e.name,
    path: t,
    language: Qa(t),
    content: i,
    dirty: !1,
    handle: e,
    size: r.size,
    source: n
  };
}
async function ta(e, t, n, r, i) {
  if (!(n.length >= i || r > 8))
    for await (const s of e.values()) {
      if (n.length >= i) break;
      if (s.kind === "directory") {
        if (hi.has(s.name)) continue;
        await ta(s, `${t}/${s.name}`, n, r + 1, i);
        continue;
      }
      const o = s, c = await ea(o, `${t}/${o.name}`, "local-folder");
      c && n.push(c);
    }
}
function er(e) {
  return new Promise((t) => {
    const n = document.createElement("input");
    n.type = "file", n.multiple = !0, n.style.display = "none", e && n.setAttribute("webkitdirectory", ""), n.onchange = async () => {
      const r = Array.from(n.files ?? []), i = await Promise.all(r.map(async (s) => {
        const o = s.webkitRelativePath || s.name;
        return _a(o, s.size) ? {
          id: tr(o),
          name: s.name,
          path: o,
          language: Qa(o),
          content: await s.text(),
          dirty: !1,
          size: s.size,
          source: e ? "local-folder" : "local-file"
        } : null;
      }));
      n.remove(), t(St(i.filter(Boolean)));
    }, document.body.append(n), n.click();
  });
}
async function An(e) {
  if (!e.handle?.createWritable)
    return vi(e), { ...e, dirty: !1 };
  const t = await e.handle.createWritable();
  return await t.write(e.content), await t.close(), { ...e, dirty: !1 };
}
function vi(e) {
  const t = new Blob([e.content], { type: "text/plain;charset=utf-8" }), n = URL.createObjectURL(t), r = document.createElement("a");
  r.href = n, r.download = e.name, r.click(), URL.revokeObjectURL(n);
}
function tr(e) {
  let t = 0;
  for (let n = 0; n < e.length; n += 1) t = t * 31 + e.charCodeAt(n) | 0;
  return `${e.replace(/[^a-z0-9]/gi, "-").toLowerCase()}-${Math.abs(t)}`;
}
function St(e) {
  return [...e].sort((t, n) => t.path.localeCompare(n.path));
}
function nr(e) {
  let t = e;
  return t = t.replace(/</g, "&lt;").replace(/>/g, "&gt;"), t = t.replace(/```(\w*)\n([\s\S]*?)```/g, (n, r, i) => `<pre style="background:var(--bg-code);padding:16px;border-radius:8px;overflow:auto;margin:12px 0"><code style="font-family:'JetBrains Mono',monospace;font-size:13px;color:var(--text-primary)">${i.trim()}</code></pre>`), t = t.replace(/`([^`]+)`/g, `<code style="background:rgba(124,77,255,0.1);padding:2px 6px;border-radius:4px;font-family:'JetBrains Mono',monospace;font-size:12px">$1</code>`), t = t.replace(/^###### (.*$)/gim, '<h6 style="font-size:13px;font-weight:600;margin:12px 0;color:var(--text-primary)">$1</h6>'), t = t.replace(/^##### (.*$)/gim, '<h5 style="font-size:14px;font-weight:600;margin:12px 0;color:var(--text-primary)">$1</h5>'), t = t.replace(/^#### (.*$)/gim, '<h4 style="font-size:16px;font-weight:600;margin:14px 0;color:var(--text-primary)">$1</h4>'), t = t.replace(/^### (.*$)/gim, '<h3 style="font-size:20px;font-weight:600;margin:16px 0;color:var(--text-primary)">$1</h3>'), t = t.replace(/^## (.*$)/gim, '<h2 style="font-size:24px;font-weight:600;margin:20px 0;padding-bottom:8px;border-bottom:1px solid var(--border-default);color:var(--text-primary)">$1</h2>'), t = t.replace(/^# (.*$)/gim, '<h1 style="font-size:32px;font-weight:700;margin:24px 0;padding-bottom:8px;border-bottom:2px solid var(--border-default);color:var(--text-primary)">$1</h1>'), t = t.replace(/\*\*\*(.+?)\*\*\*/g, "<strong><em>$1</em></strong>"), t = t.replace(/\*\*(.+?)\*\*/g, "<strong>$1</strong>"), t = t.replace(/\*(.+?)\*/g, "<em>$1</em>"), t = t.replace(/___(.+?)___/g, "<strong><em>$1</em></strong>"), t = t.replace(/__(.+?)__/g, "<strong>$1</strong>"), t = t.replace(/_(.+?)_/g, "<em>$1</em>"), t = t.replace(/~~(.+?)~~/g, "<del>$1</del>"), t = t.replace(/\[([^\]]+)\]\(([^)]+)\)/g, '<a href="$2" style="color:var(--accent-primary);text-decoration:none" target="_blank" rel="noopener">$1</a>'), t = t.replace(/!\[([^\]]*)\]\(([^)]+)\)/g, '<img src="$2" alt="$1" style="max-width:100%;border-radius:8px;margin:12px 0" />'), t = t.replace(/^&gt; (.*$)/gim, '<blockquote style="border-left:4px solid var(--accent-primary);padding-left:16px;margin:12px 0;color:var(--text-secondary)">$1</blockquote>'), t = t.replace(/^---+$/gim, '<hr style="border:none;border-top:1px solid var(--border-default);margin:24px 0" />'), t = t.replace(/^\*\*\*+$/gim, '<hr style="border:none;border-top:1px solid var(--border-default);margin:24px 0" />'), t = t.replace(/^- \[x\] (.*$)/gim, '<div style="display:flex;align-items:center;gap:8px;margin:4px 0"><span style="color:var(--accent-success)">&#9745;</span><span>$1</span></div>'), t = t.replace(/^- \[ \] (.*$)/gim, '<div style="display:flex;align-items:center;gap:8px;margin:4px 0"><span>&#9744;</span><span>$1</span></div>'), t = t.replace(/^(\d+\.\s.*(?:\n\d+\.\s.*)*)/gm, (n) => `<ol style="padding-left:24px;margin:12px 0">${n.split(`
`).map(
    (i) => `<li style="margin:4px 0">${i.replace(/^\d+\.\s/, "")}</li>`
  ).join("")}</ol>`), t = t.replace(/^([-*]\s.*(?:\n[-*]\s.*)*)/gm, (n) => `<ul style="padding-left:24px;margin:12px 0">${n.split(`
`).map(
    (i) => `<li style="margin:4px 0">${i.replace(/^[-*]\s/, "")}</li>`
  ).join("")}</ul>`), t = t.replace(/\|(.+)\|\n\|[-:\|\s]+\|\n((?:\|.+\|\n?)+)/g, (n, r, i) => {
    const s = r.split("|").filter(Boolean).map((c) => `<th style="padding:8px 12px;background:var(--bg-titlebar);font-weight:600;font-size:13px;border:1px solid var(--border-default)">${c.trim()}</th>`).join(""), o = i.trim().split(`
`).map((c) => `<tr>${c.split("|").filter(Boolean).map(
      (b, p) => `<td style="padding:8px 12px;border:1px solid var(--border-default);font-size:13px;background:${p % 2 === 0 ? "transparent" : "var(--bg-hover)"};color:var(--text-primary)">${b.trim()}</td>`
    ).join("")}</tr>`).join("");
    return `<table style="border-collapse:collapse;margin:16px 0;width:100%;border:1px solid var(--border-default)"><thead><tr>${s}</tr></thead><tbody>${o}</tbody></table>`;
  }), t = t.replace(/^(?!<[a-z])(.+)$/gim, '<p style="line-height:1.6;margin:12px 0;color:var(--text-primary)">$1</p>'), t = t.replace(/\n+/g, `
`), t;
}
const Ze = {
  kind: "atomek-ai",
  id: "atomek",
  label: "Atomek",
  description: "Workspace assistant",
  available: !0
}, yi = "atomek:selected-chat-target", cn = {
  openclaw: "OpenClaw",
  hermes: "Hermes"
}, Zt = /https?:\/\/(?:10(?:\.\d{1,3}){3}|192\.168(?:\.\d{1,3}){2}|172\.(?:1[6-9]|2\d|3[01])(?:\.\d{1,3}){2})(?::\d+)?\S*/gi, Kt = /\b(?:10(?:\.\d{1,3}){3}|192\.168(?:\.\d{1,3}){2}|172\.(?:1[6-9]|2\d|3[01])(?:\.\d{1,3}){2})\b/g, Ut = /\b(?:MiniMax(?:-M[\w.-]+)?|Moonshot|Kimi|DeepSeek|Qwen|Alibaba|Xiaomi|Nous|OpenRouter|Strato|Scalesys|ail-compound|switchAILocal|OpenAI-compatible|vLLM|llama\.cpp)\b/gi, Bt = /\b(?:route[_-]?id|pod[_-]?id|droplet[_-]?id|provider[_-]?id|publicUrl|gatewayUrl|privateUrl)\b\s*[:=]\s*[^\s,;]+/gi, Gt = /\b(?:pod-agent|ail-route|strato-eu|wannolot|scalesys)[._:-][a-z0-9._:-]{4,}\b/gi, Qt = /https?:\/\/[^\s)]*(?:tytus\.traylinx\.com|strato|scalesys|wannolot|droplet|gateway|route|\.internal|\.local|digitalocean|do\.com|hetzner)[^\s)]*/gi, $t = /\b(?:fe80|fc00|fd[0-9a-f]{2}):[0-9a-f:]+(?:%[a-z0-9]+)?\b/gi;
function ln(e) {
  return e.replace(Zt, "private gateway").replace(Qt, "private gateway").replace($t, "private gateway").replace(Kt, "private gateway").replace(Bt, "agent runtime").replace(Gt, "your Tytus pod").replace(Ut, "agent runtime").replace(/\s{3,}/g, "  ");
}
function Dn(e) {
  const t = e.toLowerCase();
  return /\b(503|502|504|warm|warming|boot|starting|not ready|temporarily unavailable)\b/.test(t) ? { message: "Agent is warming up. Try again in a moment.", retryable: !0 } : /\b(timeout|timed out|aborted)\b/.test(t) ? { message: "Connection timed out. The agent may still be working. Try again.", retryable: !0 } : /\b(offline|stopped|unreachable|not found|404)\b/.test(t) ? { message: "Agent is offline. Restart the pod or pick another agent.", retryable: !0 } : { message: ln(e).trim() || "Agent chat failed. Try again or pick another agent.", retryable: !1 };
}
function ar(e) {
  return `${yi}:${e}`;
}
function gi(e) {
  try {
    return localStorage.getItem(ar(e))?.trim() || Ze.id;
  } catch {
    return Ze.id;
  }
}
function Na(e, t) {
  try {
    localStorage.setItem(ar(e), t || Ze.id);
  } catch {
  }
}
function vt(e, ...t) {
  if (!e) return "";
  for (const n of t) {
    const r = e[n];
    if (typeof r == "string" && r.trim()) return r.trim();
    if (typeof r == "number") return String(r);
  }
  return "";
}
function wi(e) {
  const t = vt(e.metadata, "podId", "pod_id", "pod", "id");
  if (t) return t;
  const n = e.id.split(".").filter(Boolean);
  return n[0] === "pod-agent" ? n[n.length - 1] ?? e.id : n[0] === "ail-route" ? n[1] ?? e.id : e.id;
}
function ki(e) {
  return vt(e.metadata, "routeId", "route_id") || null;
}
function rr(e) {
  const t = e.toLowerCase();
  return t.includes("hermes") ? "hermes" : t.includes("openclaw") || t.includes("nemoclaw") ? "openclaw" : null;
}
function xi(e) {
  return e.kind !== "pod-agent" ? null : rr([
    e.label,
    e.id,
    vt(e.metadata, "displayName", "display_name", "name", "podName", "pod_name"),
    vt(e.metadata, "agentFamily", "agent_family", "agentType", "agent_type", "internalAgentType", "brand")
  ].join(" "));
}
function Pi(e) {
  const t = String(e.status ?? "").toLowerCase();
  return t === "ready" || t === "available" || t === "running" ? "running" : t === "degraded" || t === "starting" || t === "warming" || t === "booting" ? "warming" : t === "stopped" || t === "unreachable" || t === "needs-setup" || t === "offline" || t === "error" ? "stopped" : "unknown";
}
function Ni(e) {
  const t = e.trim();
  return t.length < 2 || t.length > 44 ? !1 : Zt.test(t) || Qt.test(t) || $t.test(t) || Kt.test(t) || Bt.test(t) || Gt.test(t) || Ut.test(t) ? (Zt.lastIndex = 0, Qt.lastIndex = 0, $t.lastIndex = 0, Kt.lastIndex = 0, Bt.lastIndex = 0, Gt.lastIndex = 0, Ut.lastIndex = 0, !1) : (Zt.lastIndex = 0, Qt.lastIndex = 0, $t.lastIndex = 0, Kt.lastIndex = 0, Bt.lastIndex = 0, Gt.lastIndex = 0, Ut.lastIndex = 0, /^[\p{L}\p{N}][\p{L}\p{N} ._()'&+-]*$/u.test(t));
}
function Ti(e, t) {
  const n = vt(e.metadata, "displayName", "display_name", "customName", "custom_name", "podDisplayName", "pod_display_name", "name");
  if (n) {
    const r = ln(n).replace(/\bNemoClaw\b/gi, "OpenClaw").replace(/\s+/g, " ").trim();
    if (Ni(r)) return r;
  }
  return cn[t];
}
function zi(e, t) {
  const n = cn[e];
  return t === "running" ? `${n} pod agent · ready` : t === "warming" ? `${n} pod agent · warming` : t === "stopped" ? `${n} pod agent · offline` : `${n} pod agent · status unknown`;
}
function sr(e) {
  const n = (e?.resources ?? []).flatMap((r) => {
    const i = xi(r);
    if (!i) return [];
    const s = Pi(r), o = wi(r);
    if (!o) return [];
    const c = ki(r);
    return [{
      kind: "pod-agent",
      id: `pod-agent:${i}:${c || o}`,
      podId: o,
      routeId: c,
      agentFamily: i,
      label: Ti(r, i),
      description: zi(i, s),
      status: s,
      available: s === "running"
    }];
  });
  return Ai(n);
}
function Ai(e) {
  const t = e.filter((s) => s.kind === "pod-agent"), n = t.reduce((s, o) => (s[o.agentFamily] = (s[o.agentFamily] ?? 0) + 1, s), { openclaw: 0, hermes: 0 }), r = {}, i = { openclaw: 0, hermes: 0 };
  return e.map((s) => {
    if (s.kind !== "pod-agent") return s;
    const o = s.label, c = cn[s.agentFamily];
    i[s.agentFamily] += 1;
    const m = o.toLowerCase();
    r[m] = (r[m] ?? 0) + 1;
    const b = t.filter((g) => g.label.toLowerCase() === m).length > 1, p = o === c && n[s.agentFamily] > 1;
    if (!b && !p) return s;
    const v = p ? i[s.agentFamily] : r[m];
    return { ...s, label: `${o} ${v}` };
  });
}
function Li(e) {
  const t = e.daemon?.state, n = t?.included ?? [], r = t?.agents ?? [], i = new Map(r.map((o) => [o.id, o])), s = n.flatMap((o) => {
    const c = o.id ?? "", b = { ...(o.agentId ? i.get(o.agentId) : void 0)?.meta ?? {}, ...o.meta ?? {} }, p = rr([c, o.kind, vt(b, "displayName", "display_name", "agentFamily", "agentType", "brand")].join(" "));
    return !c || !p ? [] : [{
      id: `pod-agent.${c}`,
      kind: "pod-agent",
      label: cn[p],
      status: o.status === "running" ? "ready" : o.status ?? "unknown",
      capabilities: ["text-gen"],
      trustTier: "tytus-pod",
      sandbox: "pod",
      allowedRoots: [],
      cost: { unit: "tytus-units", tier: "mid" },
      metadata: { ...b, podId: c }
    }];
  });
  return sr({ generatedAt: (/* @__PURE__ */ new Date()).toISOString(), resources: s });
}
async function Xi(e, t) {
  let n = null;
  if (n === null)
    try {
      n = await e.resources?.list?.() ?? null;
    } catch {
      n = null;
    }
  const r = sr(n), i = r.length > 0 ? [] : Li(e), s = /* @__PURE__ */ new Map();
  for (const o of [Ze, ...r, ...i]) s.set(o.id, o);
  return [...s.values()];
}
const na = "atomek", Ee = "atomek:default", Ci = 3e3, aa = "tytus.atomek.threadTitleOverrides", ir = "tytus.atomek.selectedThreadId", Ta = {
  available: !1,
  source: "none",
  label: "Tytus AI unavailable",
  reason: "host.ai is not available in this Tytus build."
}, ht = (e) => e.role !== "user" && e.role !== "assistant" ? null : {
  id: e.id,
  role: e.role,
  body: e.body,
  status: e.status,
  gatewayLabel: e.gatewayLabel ?? void 0,
  sourceLabel: e.role === "assistant" ? "Atomek" : void 0,
  error: e.error ?? void 0,
  createdAt: e.createdAt
}, Oi = (e, t = Ci) => e.length <= t ? e : `${e.slice(0, t)}

[...clipped ${e.length - t} chars...]`, Ln = (e, t) => {
  const n = e.split(`
`).map((r) => r.replace(/^#+\s*/, "").trim()).find(Boolean);
  return n ? n.length > 80 ? `${n.slice(0, 77)}...` : n : t;
}, Si = (e) => ["briefing", "action-list", "quiz", "plan", "storyboard", "report", "local-draft", "markdown", "memory"].includes(e) ? e : "report", za = (e) => ({
  id: e.id,
  title: e.title,
  kind: Si(e.kind),
  body: e.body,
  createdAt: e.createdAt,
  source: "ai"
}), Hi = (e) => e.length === 0 ? null : {
  kind: "workspace",
  title: "Relevant Atomek memory",
  text: e.map((t, n) => [
    `Memory ${n + 1}: ${t.title}`,
    Oi(t.body, 900)
  ].join(`
`)).join(`

---

`)
}, ra = () => {
  try {
    const e = localStorage.getItem(aa);
    if (!e) return {};
    const t = JSON.parse(e);
    return !t || typeof t != "object" ? {} : Object.fromEntries(
      Object.entries(t).filter(([, n]) => typeof n == "string" && n.trim()).map(([n, r]) => [n, String(r)])
    );
  } catch {
    return {};
  }
}, Aa = (e, t) => {
  try {
    const n = ra();
    n[e] = t, localStorage.setItem(aa, JSON.stringify(n));
  } catch {
  }
}, ji = (e) => {
  try {
    const t = ra();
    delete t[e], localStorage.setItem(aa, JSON.stringify(t));
  } catch {
  }
}, La = (e) => {
  const t = ra();
  return e.map((n) => t[n.id] ? { ...n, title: t[n.id] } : n);
}, Mi = () => {
  try {
    return localStorage.getItem(ir)?.trim() || null;
  } catch {
    return null;
  }
}, it = (e) => {
  try {
    localStorage.setItem(ir, e);
  } catch {
  }
}, an = (e) => `${Ee}:agent-session:${e}`, rn = (e) => `${Ee}:agent-transcript:${e}`, sa = (e) => `${Ee}:${na}:agent-session:${e}`, sn = (e) => `${Ee}:${na}:agent-transcript:${e}`, or = 100, Wi = (e, t) => {
  try {
    const n = localStorage.getItem(sa(e))?.trim();
    if (n) return n;
    const r = localStorage.getItem(an(e))?.trim();
    return r || t && t !== e && localStorage.getItem(an(t))?.trim() || null;
  } catch {
    return null;
  }
}, Vi = (e, t) => {
  try {
    localStorage.setItem(sa(e), t);
  } catch {
  }
}, It = (e, t) => {
  try {
    const n = localStorage.getItem(sn(e)), r = n ? JSON.parse(n) : [], i = /* @__PURE__ */ new Map();
    for (const s of [...r, ...t]) i.set(s.id, s);
    localStorage.setItem(sn(e), JSON.stringify([...i.values()].slice(-or)));
  } catch {
  }
}, _t = (e) => {
  if (!e) return [];
  const t = JSON.parse(e);
  return Array.isArray(t) ? t.filter(
    (n) => n && typeof n == "object" && n.role !== void 0 && typeof n.body == "string"
  ).slice(-or) : [];
}, Ii = (e, t) => {
  try {
    const n = _t(localStorage.getItem(sn(e)));
    if (n.length > 0) return n;
    const r = _t(localStorage.getItem(rn(e)));
    return r.length > 0 || !t || t === e ? r : _t(localStorage.getItem(rn(t)));
  } catch {
    return [];
  }
};
function Fi() {
  const e = `${Ee}:${na}:agent-transcript:`, t = [];
  try {
    for (let n = 0; n < localStorage.length; n += 1) {
      const r = localStorage.key(n);
      if (!r || !r.startsWith(e)) continue;
      const i = _t(localStorage.getItem(r));
      if (i.length === 0) continue;
      const s = i[i.length - 1], o = i.find((c) => c.role === "user");
      t.push({
        targetId: r.slice(e.length),
        lastActivityAt: typeof s?.createdAt == "number" ? s.createdAt : Date.now(),
        messageCount: i.length,
        preview: (o?.body ?? s?.body ?? "").trim().slice(0, 80)
      });
    }
  } catch {
  }
  return t.sort((n, r) => r.lastActivityAt - n.lastActivityAt);
}
function Di(e) {
  cr(e, null);
}
const cr = (e, t) => {
  try {
    localStorage.removeItem(sn(e)), localStorage.removeItem(sa(e)), localStorage.removeItem(rn(e)), localStorage.removeItem(an(e)), t && t !== e && (localStorage.removeItem(rn(t)), localStorage.removeItem(an(t)));
  } catch {
  }
}, Ri = (e) => {
  const t = e.daemon;
  return typeof t.chatAgent == "function" ? t : null;
}, Ei = (e) => {
  const t = e.map((n) => [`## ${n.title}`, n.text].filter(Boolean).join(`
`)).join(`

---

`).trim();
  return t ? [
    "Atomek workspace context follows. Use it only if relevant. Do not expose internal routing, providers, model names, private network addresses, or pod identifiers.",
    "",
    t
  ].join(`
`) : "";
}, qi = (e, t) => {
  const n = Ei(t);
  return n ? [n, "", "User message:", e].join(`
`) : e;
};
function Yi({ host: e, requestContext: t, chatSettings: n, selectedTarget: r = Ze, setStatus: i }) {
  const s = e.ai, [o, c] = T(null), [m, b] = T([]), [p, v] = T([]), [g, S] = T([]), [M, E] = T([]), [J, F] = T(Ta), [ae, me] = T(!1), W = Se(!0), Z = Se(null), K = N(async () => {
    if (!s) {
      F(Ta);
      return;
    }
    try {
      F(await s.status());
    } catch (x) {
      F({
        available: !1,
        source: "none",
        label: "Tytus AI unavailable",
        reason: x instanceof Error ? x.message : String(x)
      });
    }
  }, [s]), V = N(async (x) => {
    if (s)
      try {
        const L = await s.listArtifacts({ threadId: x });
        if (!W.current) return;
        S(L.map(za));
      } catch (L) {
        i(`AI artifacts unavailable: ${L instanceof Error ? L.message : String(L)}`);
      }
  }, [s, i]), z = N(async (x) => {
    if (s)
      try {
        const L = await s.listMessages(x);
        if (!W.current) return;
        const R = m.find((X) => X.id === x) ?? null;
        R && c(R), it(x), v(L.map(ht).filter(Boolean)), E([]), await V(x);
      } catch (L) {
        i(`Load chat failed: ${L instanceof Error ? L.message : String(L)}`);
      }
  }, [s, V, i, m]), I = N(async () => {
    if (s)
      try {
        const x = La(await s.listThreads({ workspaceKey: Ee, status: "active" })), L = Mi(), R = x.find((re) => re.id === L) ?? x[0] ?? await s.createThread({ workspaceKey: Ee, title: "Atomek chat" });
        if (!W.current) return;
        b(x[0] ? x : [R]), c(R), it(R.id);
        const X = await s.listMessages(R.id);
        if (!W.current) return;
        v(X.map(ht).filter(Boolean)), await V(R.id);
      } catch (x) {
        i(`AI chat unavailable: ${x instanceof Error ? x.message : String(x)}`);
      }
  }, [s, V, i]);
  oe(() => (W.current = !0, K(), I(), () => {
    W.current = !1;
  }), [I, K]);
  const ge = r.kind === "pod-agent" ? r.podId : null;
  oe(() => {
    if (r.kind === "pod-agent") {
      v(Ii(r.id, r.podId)), E([]);
      return;
    }
    if (!s || !o) return;
    let x = !1;
    return (async () => {
      try {
        const L = await s.listMessages(o.id);
        !x && W.current && v(L.map(ht).filter(Boolean));
      } catch {
      }
    })(), () => {
      x = !0;
    };
  }, [s, r, ge, o]);
  const ee = N(async () => {
    if (!s) return null;
    if (o) return o;
    const x = await s.createThread({ workspaceKey: Ee, title: "Atomek chat" });
    return W.current && (c(x), it(x.id), b((L) => [x, ...L.filter((R) => R.id !== x.id)])), x;
  }, [s, o]), fe = N(async () => {
    if (r.kind === "pod-agent") {
      cr(r.id, r.podId), v([]), E([]), i(`Cleared ${r.label} chat`);
      return;
    }
    if (!s) return;
    const x = await s.createThread({ workspaceKey: Ee, title: "Atomek chat" });
    c(x), it(x.id), b((L) => [x, ...L.filter((R) => R.id !== x.id)]), v([]), S([]), E([]), i("New AI chat created");
  }, [s, r, i]), Te = N(async (x, L) => {
    if (!s) return;
    const R = L.trim();
    if (R)
      try {
        const X = s.updateThread, re = m.find((ve) => ve.id === x) ?? o, he = typeof X == "function" ? await X({ threadId: x, title: R }) : { ...re ?? await ee(), id: x, title: R, updatedAt: Date.now() };
        typeof X != "function" && Aa(x, R), b((ve) => ve.map((be) => be.id === he.id ? he : be)), c((ve) => ve?.id === he.id ? he : ve), i(typeof X == "function" ? `Renamed chat: ${he.title}` : `Renamed chat locally: ${he.title}`);
      } catch (X) {
        i(`Rename chat failed: ${X instanceof Error ? X.message : String(X)}`);
      }
  }, [s, ee, i, o, m]), He = N(async (x) => {
    if (s)
      try {
        await s.deleteThread(x), ji(x);
        const L = m.filter((R) => R.id !== x);
        if (b(L), o?.id === x) {
          const R = L[0] ?? await s.createThread({ workspaceKey: Ee, title: "Atomek chat" });
          c(R), it(R.id), b((re) => re.some((he) => he.id === R.id) ? re : [R, ...re]);
          const X = await s.listMessages(R.id);
          v(X.map(ht).filter(Boolean)), await V(R.id);
        }
        i("Deleted AI chat");
      } catch (L) {
        i(`Delete chat failed: ${L instanceof Error ? L.message : String(L)}`);
      }
  }, [s, V, i, o?.id, m]), xe = N(async (x) => {
    if (!s) return null;
    try {
      const L = await ee();
      if (!L) return null;
      it(L.id);
      const R = await s.createArtifact({
        threadId: L.id,
        messageId: x.messageId ?? null,
        title: x.title?.trim() || Ln(x.body, "Atomek artifact"),
        kind: x.kind ?? "markdown",
        body: x.body
      }), X = za(R);
      return S((re) => [X, ...re.filter((he) => he.id !== X.id)]), i(`Saved AI artifact: ${X.title}`), X;
    } catch (L) {
      return i(`Save artifact failed: ${L instanceof Error ? L.message : String(L)}`), null;
    }
  }, [s, ee, i]), f = N(async (x) => {
    if (s)
      try {
        await s.deleteArtifact(x), S((L) => L.filter((R) => R.id !== x)), i("Deleted AI artifact");
      } catch (L) {
        i(`Delete artifact failed: ${L instanceof Error ? L.message : String(L)}`);
      }
  }, [s, i]), q = N(async (x) => {
    if (!s) return null;
    try {
      const L = await s.writeMemory({
        title: x.title?.trim() || Ln(x.body, "Atomek memory"),
        body: x.body,
        metadata: {
          source: "atomek",
          messageId: x.messageId ?? null
        }
      });
      return E((R) => [L, ...R.filter((X) => X.id !== L.id)].slice(0, 5)), i(`Remembered: ${L.title}`), L;
    } catch (L) {
      return i(`Remember failed: ${L instanceof Error ? L.message : String(L)}`), null;
    }
  }, [s, i]), G = N(async (x) => {
    if (!s) return [];
    const L = await s.searchMemory({ query: x, limit: 5 });
    return W.current && E(L), L;
  }, [s]), k = N(async (x, L = {}) => {
    const R = x.trim();
    if (!R) return null;
    const X = r;
    if (X.kind === "atomek-ai" && !s) return null;
    me(!0);
    let re = null;
    const he = new AbortController();
    Z.current = he;
    try {
      const ve = L.requestContext ?? t;
      if (X.kind === "pod-agent") {
        const te = Ri(e), U = Date.now(), ce = {
          id: `agent-user-${U}`,
          role: "user",
          body: R,
          status: "complete",
          createdAt: U
        }, se = `agent-assistant-${U}`, Ae = {
          id: se,
          role: "assistant",
          body: "",
          status: "streaming",
          gatewayLabel: "Tytus pod agent",
          sourceLabel: X.label,
          createdAt: U + 1
        };
        if (v((d) => [...d, ce, Ae]), It(X.id, [ce, Ae]), !X.available) {
          const d = Dn(X.description), w = { ...Ae, body: d.message, status: "error", error: d.message };
          return v((A) => A.map((C) => C.id === se ? w : C)), It(X.id, [w]), i(d.message), w;
        }
        if (!te) {
          const d = "Agent chat bridge unavailable in this Tytus host build. Update TytusOS host API/runtime to enable OpenClaw and Hermes chat.", w = { ...Ae, body: d, status: "error", error: d };
          return v((A) => A.map((C) => C.id === se ? w : C)), It(X.id, [w]), i("Agent chat bridge unavailable"), w;
        }
        let Me = "";
        const Ge = (d, w) => (v((A) => A.map((C) => C.id === se ? d : C)), It(X.id, [ce, d]), i(w), d);
        try {
          for await (const d of te.chatAgent({
            podId: X.podId,
            routeId: X.routeId ?? null,
            sessionId: Wi(X.id, X.podId),
            message: qi(R, ve),
            mode: "operator",
            target: "agent",
            modelPreference: "balanced",
            signal: he.signal
          })) {
            if (d.type === "profile") {
              const w = d.profile === "local" ? "Local Cortex" : "Cloud Cortex";
              v(
                (A) => A.map(
                  (C) => C.id === se ? { ...C, gatewayLabel: w } : C
                )
              );
            }
            if (d.type === "session" && Vi(X.id, d.sessionId), d.type === "token" && (Me = ln(`${Me}${d.text}`), v((w) => w.map((A) => A.id === se ? { ...A, body: Me, status: "streaming" } : A))), d.type === "error") {
              const w = Dn(d.message), A = { ...Ae, body: w.message, status: "error", error: w.message };
              return Ge(A, w.message);
            }
            if (d.type === "done")
              return re = { ...Ae, body: Me || "Agent finished without visible output.", status: "complete" }, Ge(re, `${X.label} answered`);
          }
        } catch (d) {
          if (he.signal.aborted || d instanceof DOMException && d.name === "AbortError")
            return re = { ...Ae, body: Me || "Stopped by user.", status: "complete" }, Ge(re, `${X.label} response stopped`);
          throw d;
        }
        return he.signal.aborted ? (re = { ...Ae, body: Me || "Stopped by user.", status: "complete" }, Ge(re, `${X.label} response stopped`)) : re || (re = { ...Ae, body: Me || "Agent finished without visible output.", status: "complete" }, Ge(re, `${X.label} answered`));
      }
      const be = await ee();
      if (!be || !s) return null;
      it(be.id);
      const Ue = (be.title ?? "").trim();
      if ((!Ue || Ue === "Atomek chat") && R.trim()) {
        const te = Ln(R, "Atomek chat");
        te && te !== Ue && (async () => {
          try {
            const U = s.updateThread, ce = typeof U == "function" ? await U({ threadId: be.id, title: te }) : { ...be, title: te, updatedAt: Date.now() };
            if (typeof U != "function" && Aa(be.id, te), !W.current) return;
            c((se) => se?.id === ce.id ? ce : se), b((se) => se.map((Ae) => Ae.id === ce.id ? ce : Ae));
          } catch {
          }
        })();
      }
      const Be = await G(R).catch(() => []), at = Hi(Be), Pe = at ? [...ve, at] : [...ve];
      let ze = null;
      for await (const te of s.sendMessage({
        threadId: be.id,
        body: R,
        gatewayPreference: n.gatewayPreference,
        model: n.model.trim() || void 0,
        context: Pe,
        signal: he.signal
      })) {
        if (te.type === "message_created") {
          const U = ht(te.message);
          if (!U) continue;
          U.role === "assistant" && (ze = U.id), v((ce) => [...ce.filter((se) => se.id !== U.id), U]);
        }
        if (te.type === "token" && (ze = te.messageId, v((U) => U.some((ce) => ce.id === te.messageId) ? U.map(
          (ce) => ce.id === te.messageId ? { ...ce, body: te.body, status: "streaming", sourceLabel: "Atomek" } : ce
        ) : [...U, {
          id: te.messageId,
          role: "assistant",
          body: te.body,
          status: "streaming",
          sourceLabel: "Atomek",
          createdAt: Date.now()
        }])), te.type === "message_updated" || te.type === "done") {
          const U = ht(te.message);
          if (!U) continue;
          U.role === "assistant" && (re = U), v((ce) => ce.map((se) => se.id === U.id ? U : se)), U.gatewayLabel && i(`AI answered via ${U.gatewayLabel}`);
        }
        if (te.type === "run_failed") {
          const U = he.signal.aborted;
          i(U ? "AI response stopped" : `AI failed: ${te.error}`), ze && v((ce) => ce.map((se) => se.id !== ze ? se : U ? { ...se, status: "complete", error: void 0, body: se.body || "Stopped by user." } : { ...se, status: "error", error: te.error, body: te.error }));
        }
      }
      const Ye = La(await s.listThreads({ workspaceKey: Ee, status: "active" }).catch(() => []));
      return W.current && Ye.length > 0 && b(Ye), K(), re;
    } catch (ve) {
      return ve instanceof DOMException && ve.name === "AbortError" ? (i("AI response stopped"), re) : (i(`AI failed: ${ve instanceof Error ? ve.message : String(ve)}`), null);
    } finally {
      Z.current === he && (Z.current = null), me(!1);
    }
  }, [s, n.gatewayPreference, n.model, ee, e, G, K, t, r, i]), O = N(() => {
    Z.current?.abort(), i("Stopping AI response…");
  }, [i]);
  return {
    aiStatus: J,
    artifacts: g,
    busy: ae,
    memoryHits: M,
    messages: p,
    thread: o,
    threads: m,
    askAgent: k,
    createArtifact: xe,
    deleteArtifact: f,
    deleteThread: He,
    newChat: fe,
    recall: G,
    remember: q,
    renameThread: Te,
    selectThread: z,
    stopChat: O
  };
}
const Ji = (e) => `tytus-workbench:///${encodeURI(e.path)}`, Zi = (e) => {
  let t = 2166136261;
  for (let n = 0; n < e.length; n += 1)
    t ^= e.charCodeAt(n), t = Math.imul(t, 16777619);
  return (t >>> 0).toString(16).padStart(8, "0");
}, Ki = ({ files: e, openEditorIds: t, activeFileId: n, versions: r, activeSelection: i }) => {
  const s = new Set(t), o = e.map((c) => {
    const m = c.id === n;
    return {
      id: c.id,
      uri: Ji(c),
      path: c.path,
      name: c.name,
      language: c.language,
      version: r[c.id] ?? 1,
      contentHash: Zi(c.content),
      dirty: c.dirty,
      source: c.source,
      selection: m && i ? i : void 0,
      updatedAt: Date.now(),
      open: s.has(c.id),
      active: m
    };
  });
  return {
    documents: o,
    byId: new Map(o.map((c) => [c.id, c])),
    activeDocumentId: n,
    openDocumentIds: t.filter((c) => o.some((m) => m.id === c))
  };
}, Ui = "active-file", Bi = (e) => e === "none" ? "No context" : e === "active-selection" ? "Active selection" : e === "active-file" ? "Active file" : e === "open-editors" ? "Open editors" : e === "selected-files" ? "Selected files" : "Indexed project", Gi = (e, t = "file") => `${t}:${e.id}`, Qi = (e, t) => [
  "selection",
  e.id,
  t.startLineNumber,
  t.startColumn,
  t.endLineNumber,
  t.endColumn
].join(":"), Ft = (e, t, n) => ({
  id: Gi(e),
  kind: "file",
  label: n ? `${n}: ${e.name}` : e.name,
  fileId: e.id,
  path: e.path,
  version: e.version,
  contentHash: e.contentHash,
  language: e.language,
  dirty: e.dirty,
  includeBody: !0,
  removable: !0,
  implicit: t
}), $i = (e, t, n) => ({
  id: Qi(e, t),
  kind: "selection",
  label: `Selection: ${e.name}:${t.startLineNumber}-${t.endLineNumber}`,
  fileId: e.id,
  path: e.path,
  range: t,
  version: e.version,
  contentHash: e.contentHash,
  language: e.language,
  dirty: e.dirty,
  includeBody: !0,
  removable: !0,
  implicit: n
}), _i = (e, t) => {
  const n = new Set(t.removedAttachmentIds), r = e.activeDocumentId ? e.byId.get(e.activeDocumentId) ?? null : null;
  let i = [];
  return t.scope === "active-selection" ? r?.selection ? i = [$i(r, r.selection, !0)] : r && (i = [Ft(r, !0, "Active file")]) : t.scope === "active-file" ? r && (i = [Ft(r, !0, "Active file")]) : t.scope === "open-editors" ? i = e.openDocumentIds.map((s) => e.byId.get(s)).filter(Boolean).map((s) => Ft(s, !0, "Open editor")) : t.scope === "selected-files" && (i = t.selectedFileIds.map((s) => e.byId.get(s)).filter(Boolean).map((s) => Ft(s, !1, "Selected file"))), eo(i).filter((s) => !n.has(s.id));
}, eo = (e) => {
  const t = /* @__PURE__ */ new Set();
  return e.filter((n) => t.has(n.id) ? !1 : (t.add(n.id), !0));
}, to = 4e3, no = 8e3, ao = 4e3, ro = (e, t) => e.length <= t ? e : `${e.slice(0, t)}

[...clipped ${e.length - t} chars...]`, so = (e, t, n) => {
  const r = _i(e, n), i = [], s = new Map(t.map((o) => [o.id, o]));
  for (const o of r) {
    if (!o.fileId) continue;
    const c = s.get(o.fileId), m = e.byId.get(o.fileId);
    if (!c || !m || !o.includeBody) continue;
    const b = [
      `Path: ${c.path}`,
      `Language: ${c.language}`,
      `Version: ${m.version}`,
      `Hash: ${m.contentHash}`,
      `Dirty: ${m.dirty ? "yes" : "no"}`,
      o.range ? `Range: ${Xa(o.range)}` : null
    ].filter(Boolean).join(`
`), p = o.kind === "selection" && o.range ? io(c.content, o.range) : c.content, v = o.kind === "selection" ? to : o.label.startsWith("Open editor") ? ao : no;
    i.push({
      kind: o.kind === "selection" ? "selection" : "file",
      title: o.kind === "selection" ? `Active selection: ${c.path}` : o.label,
      text: `${b}

${ro(p, v)}`
    });
  }
  return r.length > 0 && i.push({
    kind: "workspace",
    title: "Atomek chat context manifest",
    text: r.map((o, c) => [
      `${c + 1}. ${o.label}`,
      o.path ? `   path: ${o.path}` : null,
      o.range ? `   range: ${Xa(o.range)}` : null,
      o.version ? `   version: ${o.version}` : null,
      o.dirty ? "   dirty: yes" : null
    ].filter(Boolean).join(`
`)).join(`
`)
  }), { parts: i, attachments: r };
}, io = (e, t) => {
  const n = e.split(`
`), r = Math.max(1, t.startLineNumber), i = Math.max(r, t.endLineNumber), s = n.slice(r - 1, i);
  return s.length === 0 ? "" : s.length === 1 ? s[0].slice(Math.max(0, t.startColumn - 1), Math.max(0, t.endColumn - 1)) : (s[0] = s[0].slice(Math.max(0, t.startColumn - 1)), s[s.length - 1] = s[s.length - 1].slice(0, Math.max(0, t.endColumn - 1)), s.join(`
`));
}, Xa = (e) => `${e.startLineNumber}:${e.startColumn}-${e.endLineNumber}:${e.endColumn}`, Ca = 75e4, oo = 2800, co = 240, lo = 80, uo = /* @__PURE__ */ new Set([
  ".git",
  ".hg",
  ".svn",
  ".next",
  ".nuxt",
  ".svelte-kit",
  ".turbo",
  ".cache",
  ".parcel-cache",
  "node_modules",
  "bower_components",
  "vendor",
  "dist",
  "build",
  "coverage",
  ".vite",
  ".DS_Store"
]), ho = /* @__PURE__ */ new Set([
  "png",
  "jpg",
  "jpeg",
  "gif",
  "webp",
  "avif",
  "ico",
  "bmp",
  "tiff",
  "pdf",
  "zip",
  "gz",
  "tgz",
  "rar",
  "7z",
  "tar",
  "wasm",
  "exe",
  "dll",
  "dylib",
  "so",
  "class",
  "jar",
  "pyc",
  "pyo",
  "woff",
  "woff2",
  "ttf",
  "otf",
  "eot",
  "mp3",
  "mp4",
  "m4a",
  "mov",
  "avi",
  "webm",
  "wav",
  "flac",
  "sqlite",
  "db",
  "lock"
]), ct = (e) => {
  let t = 2166136261;
  for (let n = 0; n < e.length; n += 1)
    t ^= e.charCodeAt(n), t = Math.imul(t, 16777619);
  return (t >>> 0).toString(16).padStart(8, "0");
}, lr = (e) => ct(
  e.map((t) => `${t.id}\0${t.path}\0${t.language}\0${t.dirty ? "1" : "0"}\0${ct(t.content)}`).sort().join("")
), Rn = (e, t = {}) => {
  const n = e.size ?? e.content.length;
  return n === 0 || e.content.length === 0 ? "empty" : n > (t.maxFileBytes ?? Ca) || e.content.length > (t.maxFileBytes ?? Ca) ? "huge" : !t.includeDirty && e.dirty ? null : yo(e.path) ? "vendor" : go(e) ? "binary" : null;
}, Oa = (e, t = {}) => {
  const n = Date.now(), r = [], i = [], s = [];
  for (const o of e) {
    const c = o.size ?? o.content.length, m = Rn(o, t);
    if (m) {
      i.push({ fileId: o.id, name: o.name, path: o.path, language: o.language, reason: m, size: c });
      continue;
    }
    const b = ct(o.content), p = mo(o, b, n, t);
    s.push(...p), r.push({
      fileId: o.id,
      name: o.name,
      path: o.path,
      language: o.language,
      hash: b,
      dirty: o.dirty,
      source: o.source,
      size: c,
      chunkIds: p.map((v) => v.id),
      indexedAt: n
    });
  }
  return fo({
    indexedAt: n,
    signature: lr(e),
    files: r,
    skipped: i,
    chunks: s,
    byFileId: new Map(r.map((o) => [o.fileId, o])),
    byChunkId: new Map(s.map((o) => [o.id, o]))
  });
}, mo = (e, t = ct(e.content), n = Date.now(), r = {}) => {
  const i = Math.max(500, r.maxChunkChars ?? oo), s = Math.min(Math.max(0, r.chunkOverlapChars ?? co), Math.floor(i / 2)), o = Math.max(1, r.maxChunksPerFile ?? lo), c = bo(e.content), m = [];
  let b = 0, p = 0;
  for (; b < e.content.length && m.length < o; ) {
    const v = po(e.content, b, Math.min(e.content.length, b + i)), g = e.content.slice(b, v).trim();
    if (g.length > 0) {
      const S = vo(c, b, v);
      m.push({
        id: `${e.id}:${t}:${p}:${S.startLineNumber}-${S.endLineNumber}`,
        fileId: e.id,
        path: e.path,
        name: e.name,
        language: e.language,
        hash: t,
        dirty: e.dirty,
        source: e.source,
        size: e.size ?? e.content.length,
        ordinal: p,
        text: g,
        range: S,
        charStart: b,
        charEnd: v,
        indexedAt: n
      }), p += 1;
    }
    if (v >= e.content.length) break;
    b = Math.max(b + 1, v - s);
  }
  return m;
}, fo = (e) => ({
  ...e,
  files: Object.freeze([...e.files]),
  skipped: Object.freeze([...e.skipped]),
  chunks: Object.freeze([...e.chunks])
}), po = (e, t, n) => {
  if (n >= e.length) return e.length;
  const r = Math.max(t + 1, n - 500), i = e.lastIndexOf(`
`, n);
  if (i >= r) return i + 1;
  const s = Math.max(e.lastIndexOf(". ", n), e.lastIndexOf("; ", n));
  if (s >= r) return s + 1;
  const o = e.lastIndexOf(" ", n);
  return o >= r ? o + 1 : n;
}, bo = (e) => {
  const t = [0];
  for (let n = 0; n < e.length; n += 1)
    e[n] === `
` && t.push(n + 1);
  return t;
}, vo = (e, t, n) => {
  const r = Sa(e, t), i = Sa(e, Math.max(t, n - 1));
  return {
    startLineNumber: r + 1,
    startColumn: t - e[r] + 1,
    endLineNumber: i + 1,
    endColumn: n - e[i] + 1
  };
}, Sa = (e, t) => {
  let n = 0, r = e.length - 1;
  for (; n <= r; ) {
    const i = Math.floor((n + r) / 2);
    if (e[i] <= t && (i === e.length - 1 || e[i + 1] > t)) return i;
    e[i] > t ? r = i - 1 : n = i + 1;
  }
  return 0;
}, yo = (e) => e.split(/[\\/]+/).filter(Boolean).some((t) => uo.has(t)), go = (e) => {
  const t = e.name.includes(".") ? e.name.split(".").pop()?.toLowerCase() : void 0;
  if (t && ho.has(t) || e.content.includes("\0")) return !0;
  const n = e.content.slice(0, 4096);
  if (!n) return !1;
  let r = 0;
  for (let i = 0; i < n.length; i += 1) {
    const s = n.charCodeAt(i);
    (s < 9 || s > 13 && s < 32) && (r += 1);
  }
  return r / n.length > 0.08;
};
class wo {
  snapshot;
  options;
  constructor(t = [], n = {}) {
    this.options = { ...n }, this.snapshot = Oa(t, this.options);
  }
  getSnapshot() {
    return this.snapshot;
  }
  getOptions() {
    return { ...this.options };
  }
  refresh(t, n = this.options) {
    return this.options = { ...n }, this.snapshot = Oa(t, this.options), this.snapshot;
  }
  update(t, n = this.options) {
    return this.refresh(t, n);
  }
  staleReport(t) {
    return ko(this.snapshot, t, this.options);
  }
  isStale(t) {
    return this.staleReport(t).stale;
  }
}
const mt = (e = [], t = {}) => new wo(e, t), ko = (e, t, n = {}) => {
  const r = new Map(t.map((c) => [c.id, c])), i = e.byFileId, s = [];
  for (const c of e.files) {
    const m = r.get(c.fileId);
    if (!m) {
      s.push({ fileId: c.fileId, path: c.path, status: "deleted", indexedHash: c.hash });
      continue;
    }
    if (Rn(m, n)) {
      s.push({ fileId: c.fileId, path: m.path, status: "skipped-now", indexedHash: c.hash, currentHash: ct(m.content) });
      continue;
    }
    const p = ct(m.content);
    p !== c.hash ? s.push({ fileId: c.fileId, path: m.path, status: "changed", indexedHash: c.hash, currentHash: p }) : m.dirty !== c.dirty && s.push({ fileId: c.fileId, path: m.path, status: "dirty-state-changed", indexedHash: c.hash, currentHash: p });
  }
  for (const c of t)
    !i.has(c.id) && !Rn(c, n) && s.push({ fileId: c.id, path: c.path, status: "new", currentHash: ct(c.content) });
  const o = e.signature !== lr(t);
  return { stale: s.length > 0, signatureChanged: o, files: s };
}, xo = 8, Po = 12e3, dr = (e, t, n = {}, r) => {
  const i = zo(t);
  if (i.length === 0) return [];
  const s = new Set(r?.files.map((p) => p.fileId) ?? []), o = e.chunks.filter((p) => n.includeDirty !== !1 || !p.dirty).map((p) => Ao(p, i)).filter((p) => p.score >= (n.minScore ?? 1)).sort((p, v) => v.score - p.score || p.chunk.path.localeCompare(v.chunk.path) || p.chunk.ordinal - v.chunk.ordinal), c = [], m = /* @__PURE__ */ new Set();
  let b = n.maxChars ?? Po;
  for (const p of o) {
    if (c.length >= (n.limit ?? xo) || b <= 0) break;
    const v = `${p.chunk.fileId}:${p.chunk.ordinal}`;
    if (m.has(v)) continue;
    m.add(v);
    const g = p.chunk.text.length > b ? `${p.chunk.text.slice(0, Math.max(0, b - 28))}
[...context clipped...]` : p.chunk.text;
    c.push(No(p, g, s.has(p.chunk.fileId))), b -= g.length;
  }
  return c;
}, No = (e, t = e.chunk.text, n = !1) => {
  const { chunk: r, score: i, matchedTerms: s } = e;
  return {
    id: `index-hit:${r.id}`,
    kind: "index-hit",
    label: `${r.path}:${r.range.startLineNumber}-${r.range.endLineNumber}`,
    fileId: r.fileId,
    chunkId: r.id,
    path: r.path,
    range: r.range,
    language: r.language,
    contentHash: r.hash,
    dirty: r.dirty,
    includeBody: !0,
    removable: !0,
    implicit: !1,
    stale: n,
    charCount: t.length,
    text: t,
    snippet: Lo(r.text, s),
    score: i
  };
}, To = (e) => [
  `Path: ${e.path}`,
  `Language: ${e.language}`,
  `Range: ${e.range.startLineNumber}:${e.range.startColumn}-${e.range.endLineNumber}:${e.range.endColumn}`,
  `Hash: ${e.contentHash}`,
  e.dirty ? "Dirty: yes" : "Dirty: no",
  e.stale ? "Stale: yes" : null,
  "",
  e.text
].filter((t) => t !== null).join(`
`), zo = (e) => Array.from(new Set(
  e.toLowerCase().split(/[^a-z0-9_.$/-]+/i).map((t) => t.trim()).filter((t) => t.length >= 2)
)), Ao = (e, t) => {
  const n = e.text.toLowerCase(), r = e.path.toLowerCase(), i = e.name.toLowerCase(), s = e.language.toLowerCase();
  let o = 0;
  const c = [];
  for (const m of t) {
    const b = Xn(n, m), p = Xn(r, m), v = Xn(i, m), g = s === m ? 1 : 0;
    b + p + v + g !== 0 && (c.push(m), o += Math.min(b, 12), o += p * 4, o += v * 6, o += g * 3, (n.includes(`function ${m}`) || n.includes(`const ${m}`) || n.includes(`class ${m}`) || n.includes(`type ${m}`)) && (o += 3));
  }
  return o += Math.max(0, 2 - e.ordinal * 0.05), { chunk: e, score: o, matchedTerms: c };
}, Xn = (e, t) => {
  let n = 0, r = e.indexOf(t);
  for (; r !== -1; )
    n += 1, r = e.indexOf(t, r + t.length);
  return n;
}, Lo = (e, t) => {
  const n = e.toLowerCase(), r = t.map((m) => n.indexOf(m)).filter((m) => m >= 0).sort((m, b) => m - b)[0] ?? 0, i = Math.max(0, r - 120), s = Math.min(e.length, r + 260), o = i > 0 ? "…" : "", c = s < e.length ? "…" : "";
  return `${o}${e.slice(i, s).replace(/\s+/g, " ").trim()}${c}`;
}, Xo = (e, t = {}) => {
  const { autoRefresh: n = !1, ...r } = t, i = Se(e), s = Se(null);
  s.current || (s.current = mt(e, r));
  const [o, c] = T(() => s.current?.getSnapshot() ?? mt(e, r).getSnapshot());
  i.current = e;
  const m = N((g = i.current) => {
    const S = (s.current ?? mt([], r)).refresh(g, r);
    return c(S), S;
  }, [r.maxFileBytes, r.maxChunkChars, r.chunkOverlapChars, r.maxChunksPerFile, r.includeDirty]), b = N((g = i.current) => {
    const S = (s.current ?? mt([], r)).update(g, r);
    return c(S), S;
  }, [m]);
  oe(() => {
    n && m(e);
  }, [n, e, m]);
  const p = (s.current ?? mt([], r)).staleReport(e), v = N((g, S = {}) => {
    const M = s.current ?? mt([], r);
    return dr(M.getSnapshot(), g, S, M.staleReport(i.current));
  }, []);
  return { snapshot: o, staleReport: p, isStale: p.stale, refresh: m, update: b, retrieve: v };
}, Co = "tytus.atomek.semanticVector:v1", ur = "__gateway_default__", hr = (e) => e?.trim() || ur, Oo = (e) => {
  const t = hr(e);
  return t === ur ? "gateway default" : t;
}, mr = (e, t, n) => [
  Co,
  encodeURIComponent(e || "unknown-app"),
  encodeURIComponent(t),
  encodeURIComponent(n.id),
  n.hash
].join(":"), fr = () => {
  try {
    return typeof window < "u" && typeof window.localStorage < "u";
  } catch {
    return !1;
  }
}, So = (e, t, n) => {
  if (!fr()) return null;
  try {
    const r = mr(e, t, n), i = window.localStorage.getItem(r);
    if (!i) return null;
    const s = JSON.parse(i);
    return s.chunkId !== n.id || s.contentHash !== n.hash || s.modelAlias !== t || !Array.isArray(s.vector) || s.vector.some((o) => typeof o != "number" || !Number.isFinite(o)) ? null : {
      key: r,
      appId: e,
      chunkId: n.id,
      contentHash: n.hash,
      modelAlias: t,
      model: typeof s.model == "string" ? s.model : "",
      gatewayLabel: typeof s.gatewayLabel == "string" ? s.gatewayLabel : "",
      dim: s.vector.length,
      vector: s.vector,
      updatedAt: typeof s.updatedAt == "number" ? s.updatedAt : 0
    };
  } catch {
    return null;
  }
}, Ho = (e) => {
  if (!fr()) return null;
  try {
    const t = mr(e.appId, e.modelAlias, { id: e.chunkId, hash: e.contentHash }), n = { ...e, key: t };
    return window.localStorage.setItem(t, JSON.stringify(n)), n;
  } catch {
    return null;
  }
}, zt = 8, jo = 12e3, Mo = 180, Wo = async (e, t, n, r, i = {}, s, o) => {
  const c = dr(t, n, { ...i, limit: Math.max(i.limit ?? zt, 16) }, s), m = e.ai;
  if (typeof m?.embedText != "function")
    return { hits: c.slice(0, i.limit ?? zt), mode: "keyword", reason: "Semantic index unavailable — using keyword retrieval.", embeddedChunks: 0 };
  try {
    const b = hr(r.embeddingModel), p = await m.embedText({
      input: n,
      gatewayPreference: r.gatewayPreference,
      model: r.embeddingModel.trim() || void 0,
      signal: o
    }), v = pr(p.embedding);
    if (!v) throw new Error("host.ai.embedText returned no embedding vector");
    const g = new Map(c.map((W) => [W.chunkId, W])), S = qo(n), M = /* @__PURE__ */ new Map();
    t.chunks.filter((W) => i.includeDirty !== !1 || !W.dirty).slice(0, Mo).forEach((W) => M.set(W.id, W)), c.forEach((W) => {
      const Z = t.byChunkId.get(W.chunkId);
      Z && M.set(Z.id, Z);
    });
    const E = Array.from(M.values()), J = [];
    let F = 0;
    for (const W of E) {
      o?.aborted;
      const Z = So(e.appId, b, W) ?? await Io(m, e.appId, b, W, r, o);
      if (!Z) continue;
      Z.updatedAt > Date.now() - 1e3 && (F += 1);
      const K = Ro(v, Z.vector);
      if (!Number.isFinite(K)) continue;
      const z = g.get(W.id)?.score ?? Yo(W, S), I = Eo(z) * 0.42 + Math.max(0, K) * 0.58;
      J.push({ chunk: W, keywordScore: z, vectorScore: K, score: I, matchedTerms: S.filter((ge) => W.text.toLowerCase().includes(ge)) });
    }
    if (J.length === 0) return { hits: c.slice(0, i.limit ?? zt), mode: "vector-fallback", reason: "Semantic retrieval produced no vectors — using keyword retrieval.", embeddedChunks: F };
    const ae = new Set(s?.files.map((W) => W.fileId) ?? []), me = J.sort((W, Z) => Z.score - W.score || Z.keywordScore - W.keywordScore || W.chunk.path.localeCompare(Z.chunk.path) || W.chunk.ordinal - Z.chunk.ordinal).slice(0, Math.max(i.limit ?? zt, 1)).map((W) => Fo(W, ae.has(W.chunk.fileId)));
    return {
      hits: Do(me, i.maxChars ?? jo),
      mode: "hybrid",
      reason: `Hybrid retrieval used ${Oo(r.embeddingModel)} embeddings + keyword ranking.`,
      embeddedChunks: F
    };
  } catch (b) {
    return {
      hits: c.slice(0, i.limit ?? zt),
      mode: "vector-fallback",
      reason: `Semantic retrieval failed (${b instanceof Error ? b.message : String(b)}) — using keyword retrieval.`,
      embeddedChunks: 0
    };
  }
}, Vo = (e) => e.map((t) => ({
  kind: "workspace",
  title: `Indexed project context — ${t.label}`,
  text: To(t)
})), Io = async (e, t, n, r, i, s) => {
  if (typeof e.embedText != "function") return null;
  const o = await e.embedText({
    input: r.text,
    gatewayPreference: i.gatewayPreference,
    model: i.embeddingModel.trim() || void 0,
    signal: s
  }), c = pr(o.embedding);
  return c ? Ho({
    appId: t,
    chunkId: r.id,
    contentHash: r.hash,
    modelAlias: n,
    model: o.model ?? "",
    gatewayLabel: o.gatewayLabel ?? "",
    dim: c.length,
    vector: c,
    updatedAt: Date.now()
  }) : null;
}, Fo = (e, t = !1) => {
  const n = `${e.chunk.path}:${e.chunk.range.startLineNumber}-${e.chunk.range.endLineNumber}`;
  return {
    id: `index-hit:${e.chunk.id}`,
    kind: "index-hit",
    label: n,
    fileId: e.chunk.fileId,
    chunkId: e.chunk.id,
    path: e.chunk.path,
    range: e.chunk.range,
    language: e.chunk.language,
    contentHash: e.chunk.hash,
    dirty: e.chunk.dirty,
    includeBody: !0,
    removable: !0,
    implicit: !1,
    stale: t,
    charCount: e.chunk.text.length,
    text: e.chunk.text,
    snippet: Jo(e.chunk.text, e.matchedTerms),
    score: e.score,
    keywordScore: e.keywordScore,
    vectorScore: e.vectorScore
  };
}, Do = (e, t) => {
  let n = t;
  const r = [];
  for (const i of e) {
    if (n <= 0) break;
    const s = i.text.length > n ? `${i.text.slice(0, Math.max(0, n - 28))}
[...context clipped...]` : i.text;
    r.push({ ...i, text: s, charCount: s.length }), n -= s.length;
  }
  return r;
}, pr = (e) => {
  if (!Array.isArray(e) || e.length === 0) return null;
  const t = e.map(Number).filter((n) => Number.isFinite(n));
  return t.length === e.length ? t : null;
}, Ro = (e, t) => {
  const n = Math.min(e.length, t.length);
  if (n === 0) return 0;
  let r = 0, i = 0, s = 0;
  for (let o = 0; o < n; o += 1)
    r += e[o] * t[o], i += e[o] * e[o], s += t[o] * t[o];
  return i === 0 || s === 0 ? 0 : r / (Math.sqrt(i) * Math.sqrt(s));
}, Eo = (e) => Math.min(1, Math.max(0, e / 24)), qo = (e) => Array.from(new Set(
  e.toLowerCase().split(/[^a-z0-9_.$/-]+/i).map((t) => t.trim()).filter((t) => t.length >= 2)
)), Yo = (e, t) => {
  const n = `${e.path}
${e.name}
${e.language}
${e.text}`.toLowerCase();
  return t.reduce((r, i) => r + (n.includes(i) ? 1 : 0), 0);
}, Jo = (e, t) => {
  const n = e.toLowerCase(), r = t.map((o) => n.indexOf(o)).filter((o) => o >= 0).sort((o, c) => o - c)[0] ?? 0, i = Math.max(0, r - 120), s = Math.min(e.length, r + 260);
  return `${i > 0 ? "…" : ""}${e.slice(i, s).replace(/\s+/g, " ").trim()}${s < e.length ? "…" : ""}`;
}, Zo = /```([^\n`]*)\n([\s\S]*?)```/g, Ko = /^@@\s+-(\d+)(?:,\d+)?\s+\+(\d+)(?:,\d+)?\s+@@/;
function ia(e) {
  return Array.from(e.matchAll(Zo)).map((t) => br(t[1] ?? "", $o(t[2] ?? ""))).filter((t) => t.content.trim().length > 0);
}
function br(e, t) {
  const n = e.trim().split(/[\s,]+/).filter(Boolean), r = {}, i = [];
  for (const s of n) {
    const o = s.match(/^([a-z0-9_-]+)=(.+)$/i);
    o ? r[o[1].toLowerCase()] = ec(o[2]) : i.push(s.toLowerCase());
  }
  return { lang: e.trim().toLowerCase(), flags: i, attrs: r, content: t };
}
function Re(e, t) {
  return (typeof e == "string" ? br(e, "").flags : e.flags).includes(t.toLowerCase());
}
function vr(e) {
  return [...ia(e).filter((r) => Re(r, "diff") || Re(r, "patch")).map((r) => r.content), e].filter((r, i, s) => Uo(r) && s.findIndex((o) => o === r) === i);
}
function Uo(e) {
  return /^@@\s+-\d+/m.test(e) || /^diff --git\s+/m.test(e) || /^---\s+/m.test(e);
}
function Bo(e) {
  const t = yr(e);
  if (!/^@@\s+-\d+/m.test(t)) return [];
  const n = t.split(`
`), r = [];
  if (n.forEach((o, c) => {
    (o.startsWith("diff --git ") || o.startsWith("--- ")) && r.push(c);
  }), r.length === 0) return [{ raw: t, ...Cn(t) }];
  const i = Array.from(new Set(r)).sort((o, c) => o - c), s = [];
  for (let o = 0; o < i.length; o += 1) {
    const c = i[o], b = i.find((v) => v > c && n[v].startsWith("diff --git ")) ?? n.length, p = n.slice(c, b).join(`
`);
    /^@@\s+-\d+/m.test(p) && s.push({ raw: p, ...Cn(p) });
  }
  return s.length > 0 ? s : [{ raw: t, ...Cn(t) }];
}
function Cn(e) {
  const t = e.match(/^diff --git\s+(?:"?a\/(.+?)"?|(\S+))\s+(?:"?b\/(.+?)"?|(\S+))/m), n = e.match(/^---\s+(?:"?a\/(.+?)"?|(\S+))/m), r = e.match(/^\+\+\+\s+(?:"?b\/(.+?)"?|(\S+))/m), i = $e(t?.[1] || t?.[2] || n?.[1] || n?.[2] || "") || null, s = $e(t?.[3] || t?.[4] || r?.[1] || r?.[2] || "") || null, o = s && s !== "/dev/null" ? s : i;
  return { path: o && o !== "/dev/null" ? o : null, oldPath: i, newPath: s };
}
function $e(e) {
  return e.trim().replace(/^['"]|['"]$/g, "").replace(/\\/g, "/").replace(/^[ab]\//, "").replace(/^\.\//, "");
}
function En(e) {
  return ia(e).filter((t) => Go(t)).map((t) => ({
    content: _o(t.content),
    label: `replacement block (${t.lang || "plain"})`,
    lang: t.lang,
    path: Qo(t),
    attrs: t.attrs
  }));
}
function Go(e) {
  return Re(e, "atomek-replace") || Re(e, "atomek-full") || Re(e, "full-replacement") || Re(e, "full") || Re(e, "replace");
}
function Qo(e) {
  const t = e.attrs.path || e.attrs.file || e.attrs.target;
  if (t) return $e(t);
  const n = e.content.split(`
`).slice(0, 5);
  for (const r of n) {
    const i = r.match(/^\s*(?:\/\/|#|<!--)?\s*(?:atomek-)?(?:path|file)\s*:\s*([^\s>]+)\s*(?:-->)?\s*$/i);
    if (i) return $e(i[1]);
  }
  return null;
}
function qn(e, t) {
  if (!/^@@\s+-\d+/m.test(t)) return null;
  const n = e.split(`
`), r = yr(t).split(`
`), i = [];
  let s = 0, o = !1;
  for (let c = 0; c < r.length; c += 1) {
    const m = r[c].match(Ko);
    if (!m) continue;
    o = !0;
    const b = Number(m[1]), p = Math.max(0, b - 1);
    if (p < s) return null;
    for (i.push(...n.slice(s, p)), s = p, c += 1; c < r.length; c += 1) {
      const v = r[c];
      if (v.startsWith("@@ ")) {
        c -= 1;
        break;
      }
      if (v.startsWith("diff --git ") || v.startsWith("--- ") || v.startsWith("+++ ") || v.startsWith("\\ No newline at end of file")) continue;
      const g = v[0], S = v.slice(1);
      if (g === " ") {
        if (n[s] !== S) return null;
        i.push(S), s += 1;
        continue;
      }
      if (g === "-") {
        if (n[s] !== S) return null;
        s += 1;
        continue;
      }
      if (g === "+") {
        i.push(S);
        continue;
      }
      if (v !== "")
        return null;
    }
  }
  return o ? (i.push(...n.slice(s)), i.join(`
`)) : null;
}
function $o(e) {
  return e.replace(/^\n+/, "").replace(/\n+$/, "");
}
function _o(e) {
  const t = e.split(`
`);
  let n = 0;
  for (; n < Math.min(t.length, 5) && /^\s*(?:\/\/|#|<!--)?\s*(?:atomek-)?(?:path|file|version|hash)\s*:/i.test(t[n]); )
    n += 1;
  return t.slice(n).join(`
`);
}
function yr(e) {
  return e.replace(/\r\n/g, `
`).replace(/\r/g, `
`);
}
function ec(e) {
  return e.replace(/^['"]|['"]$/g, "");
}
function tc(e) {
  const { body: t, files: n, sourceTitle: r, activeFile: i = null, versions: s = {} } = e, o = /* @__PURE__ */ new Map(), c = [];
  for (const p of vr(t))
    for (const v of Bo(p)) {
      if (!v.path) {
        if (i) {
          const M = qn(i.content, v.raw);
          M && M !== i.content && o.set(i.id, Dt({
            file: i,
            proposedContent: M,
            sourceTitle: r,
            extractionLabel: "active-file unified diff patch",
            versions: s
          }));
        }
        continue;
      }
      const g = Ha(n, v.path);
      if (!g) {
        c.push(`${v.path}: no opened file`);
        continue;
      }
      const S = qn(g.file.content, v.raw);
      if (!S || S === g.file.content) {
        c.push(`${v.path}: patch did not match or produced no change`);
        continue;
      }
      o.set(g.file.id, Dt({
        file: g.file,
        proposedContent: S,
        sourceTitle: r,
        extractionLabel: `workspace diff (${v.path})`,
        versions: s,
        targetPath: v.path,
        match: g
      }));
    }
  for (const p of En(t)) {
    const v = p.path ? Ha(n, p.path) : i ? { file: i, normalizedPatchPath: $e(i.path), confidence: "exact" } : null;
    if (!v) {
      c.push(`${p.path ?? "replacement block"}: no opened file`);
      continue;
    }
    if (p.content === v.file.content) {
      c.push(`${v.file.path}: replacement produced no change`);
      continue;
    }
    o.set(v.file.id, Dt({
      file: v.file,
      proposedContent: p.content,
      sourceTitle: r,
      extractionLabel: p.label,
      versions: s,
      targetPath: p.path ?? v.file.path,
      match: v
    }));
  }
  const m = En(t).some((p) => p.path);
  if (o.size === 0 && i && !m) {
    const p = nc(t, i);
    p && p.content !== i.content && o.set(i.id, Dt({
      file: i,
      proposedContent: p.content,
      sourceTitle: r,
      extractionLabel: p.label,
      versions: s
    }));
  }
  const b = Array.from(o.values());
  return {
    sourceTitle: r,
    edits: b,
    skipped: Array.from(new Set(c)),
    stats: rc(b.map((p) => p.stats)),
    kind: b.length === 0 ? "empty" : b.length === 1 ? "single-file" : "multi-file"
  };
}
function nc(e, t) {
  for (const m of vr(e)) {
    const b = qn(t.content, m);
    if (b) return { content: b, label: "unified diff patch" };
  }
  const n = ia(e);
  if (n.length === 0) return null;
  const r = ic(t), i = En(e)[0];
  if (i) return { content: i.content, label: i.label };
  const s = n.find((m) => r.some((b) => Re(m, b)) && !Re(m, "diff") && !Re(m, "patch"));
  if (s) return { content: s.content, label: `matched ${s.lang || t.language} block` };
  const c = n.filter((m) => !Re(m, "diff") && !Re(m, "patch")).sort((m, b) => b.content.length - m.content.length)[0];
  return c ? { content: c.content, label: `largest fenced block (${c.lang || "plain"})` } : null;
}
function Ha(e, t) {
  const n = $e(t), r = n.split("/").at(-1) ?? n, i = e.find((c) => $e(c.path) === n);
  if (i) return { file: i, normalizedPatchPath: n, confidence: "exact" };
  const s = e.find((c) => $e(c.path).endsWith(`/${n}`));
  if (s) return { file: s, normalizedPatchPath: n, confidence: "suffix" };
  const o = e.find((c) => c.name === r || $e(c.path).endsWith(`/${r}`));
  return o ? { file: o, normalizedPatchPath: n, confidence: "basename" } : null;
}
function ac(e, t) {
  const n = e.split(`
`), r = t.split(`
`), i = Math.max(n.length, r.length);
  let s = 0, o = 0, c = 0;
  for (let m = 0; m < i; m += 1)
    n[m] !== r[m] && (n[m] === void 0 ? s += 1 : r[m] === void 0 ? o += 1 : c += 1);
  return { added: s, removed: o, changed: c };
}
function rc(e) {
  return e.reduce((t, n) => ({
    added: t.added + n.added,
    removed: t.removed + n.removed,
    changed: t.changed + n.changed
  }), { added: 0, removed: 0, changed: 0 });
}
function sc(e) {
  let t = 2166136261;
  for (let n = 0; n < e.length; n += 1)
    t ^= e.charCodeAt(n), t = Math.imul(t, 16777619);
  return (t >>> 0).toString(16).padStart(8, "0");
}
function Dt(e) {
  const t = sc(e.file.content);
  return {
    fileId: e.file.id,
    fileName: e.file.name,
    filePath: e.file.path,
    originalContent: e.file.content,
    proposedContent: e.proposedContent,
    sourceTitle: e.sourceTitle,
    extractionLabel: e.extractionLabel,
    stats: ac(e.file.content, e.proposedContent),
    base: {
      version: e.versions[e.file.id],
      contentHash: t
    },
    targetPath: e.targetPath,
    match: e.match ? { normalizedPatchPath: e.match.normalizedPatchPath, confidence: e.match.confidence } : void 0,
    conflict: {
      expectedVersion: e.versions[e.file.id],
      currentVersion: e.versions[e.file.id],
      expectedHash: t,
      currentHash: t,
      changedAfterPreview: !1
    }
  };
}
function ic(e) {
  const t = e.name.split(".").pop()?.toLowerCase() ?? "";
  return Array.from(new Set([
    e.language,
    t,
    e.language === "typescript" ? "ts" : "",
    e.language === "javascript" ? "js" : "",
    e.language === "markdown" ? "md" : "",
    e.language === "shell" ? "sh" : "",
    e.language === "yaml" ? "yml" : ""
  ].filter(Boolean)));
}
const dn = (e) => typeof e == "object" && e !== null, oc = (e) => dn(e.ai) ? e.ai : null, cc = (e) => {
  if (!dn(e)) return null;
  const t = e.id;
  return typeof t == "string" && t.trim() ? t : null;
}, lc = (e) => {
  const t = cc(e);
  return !t || !dn(e) ? null : { ...e, id: t };
}, Je = (e) => typeof e == "string" ? /\bembeddings?\b|text-embedding/i.test(e) : Array.isArray(e) ? e.some(Je) : dn(e) ? Object.values(e).some(Je) : !1, dc = async (e, t) => {
  const n = oc(e);
  if (typeof n?.listModels != "function") return [];
  const r = await n.listModels(t);
  return Array.isArray(r) ? r.map(lc).filter((i) => !!i) : [];
}, uc = (e) => (e.embedding ?? e.embeddings ?? e.supportsEmbedding ?? e.supportsEmbeddings) === !0 ? !0 : Je(e.capability) || Je(e.capabilities) || Je(e.modality) || Je(e.modalities) || Je(e.task) || Je(e.tasks) || Je(e.type) || Je(e.kind), gr = (e) => typeof e == "object" && e !== null, hc = (e) => gr(e.ai) ? e.ai : null, wr = (e) => typeof hc(e)?.embedText == "function", mc = async (e, t) => wr(e) ? (await dc(e, t)).filter(uc) : [], fc = (e) => gr(e.ai) ? wr(e) ? null : "host.ai.embedText is not exposed by this Tytus build." : "host.ai is not available in this Tytus build.", pc = ["typecheck", "test", "lint", "build", "release:check", "verify", "verify:cortex"], bc = [
  ["package-lock.json", "npm"],
  ["pnpm-lock.yaml", "pnpm"],
  ["yarn.lock", "yarn"],
  ["bun.lockb", "bun"],
  ["bun.lock", "bun"]
];
function vc(e, t) {
  return {
    id: `manual-check-${Date.now()}`,
    reason: t,
    commands: xc(e),
    results: [],
    createdAt: Date.now()
  };
}
function yc(e, t) {
  const n = kr(t);
  return !n || e.commands.some((r) => r.command === n) ? e : {
    ...e,
    commands: [
      ...e.commands,
      {
        id: Pr(n),
        command: n,
        label: n,
        source: "manual"
      }
    ]
  };
}
function gc(e, t, n, r) {
  const i = kr(t);
  return i ? {
    ...e,
    results: [
      ...e.results,
      {
        command: i,
        status: n,
        output: r.trim(),
        capturedAt: Date.now()
      }
    ]
  } : e;
}
function wc(e) {
  const t = e.results.length > 0 ? e.results.map((r, i) => [
    `Check ${i + 1}: ${r.command}`,
    `Status: ${r.status}`,
    "Output:",
    Lc(r.output || "(no output pasted)", "text")
  ].join(`
`)).join(`

`) : "No manual check output was captured yet.", n = e.commands.length > 0 ? e.commands.map((r) => `- ${r.command}`).join(`
`) : "- No check command was suggested; user must provide one manually.";
  return [
    "Continue the agentic edit/check loop from a manual check capture.",
    "Do not assume host command execution exists. The user ran or will run checks outside Atomek.",
    "Use only the currently attached workbench context and the pasted output below.",
    "If a fix is needed, return one applicable git-style unified diff in a fenced diff block with paths matching opened files.",
    "Do not write files, do not invoke tools, and do not assume any provider-specific model/tool.",
    "",
    `Manual check reason: ${e.reason}`,
    "",
    "Available manual check commands:",
    n,
    "",
    "Captured manual check results:",
    t
  ].join(`
`);
}
function kc(e) {
  const t = e.results.at(-1);
  return t ? t.status : "pending";
}
function xc(e) {
  const t = e.filter((r) => r.name === "package.json" || r.path.endsWith("/package.json")), n = [];
  for (const r of t) {
    const i = Pc(r.content);
    if (!i?.scripts) continue;
    const s = xr(r.path), o = Nc(e, s, i);
    if (!o) continue;
    const c = Object.keys(i.scripts).filter((b) => typeof i.scripts?.[b] == "string"), m = Tc(c);
    for (const b of m) {
      const p = `${o} run ${b}`;
      n.push({
        id: Pr(`${s}:${p}`),
        command: p,
        label: s ? `${b} (${s})` : b,
        source: "package-script",
        path: r.path
      });
    }
  }
  return zc(n).slice(0, 6);
}
function Pc(e) {
  try {
    const t = JSON.parse(e);
    return t && typeof t == "object" ? t : null;
  } catch {
    return null;
  }
}
function Nc(e, t, n) {
  if (typeof n.packageManager == "string") {
    const r = n.packageManager.split("@")[0]?.trim();
    if (r) return r;
  }
  for (const [r, i] of bc)
    if (e.some((s) => Ac(s.path) === r && xr(s.path) === t)) return i;
  return null;
}
function Tc(e) {
  const t = pc.filter((r) => e.includes(r)), n = e.filter((r) => !t.includes(r)).filter((r) => /(^|:)(check|typecheck|test|lint|verify|build)(:|$)/i.test(r)).sort((r, i) => r.localeCompare(i));
  return [...t, ...n];
}
function zc(e) {
  const t = /* @__PURE__ */ new Set();
  return e.filter((n) => {
    const r = n.command;
    return t.has(r) ? !1 : (t.add(r), !0);
  });
}
function kr(e) {
  return e.trim().replace(/\s+/g, " ");
}
function xr(e) {
  const t = e.lastIndexOf("/");
  return t > 0 ? e.slice(0, t) : "";
}
function Ac(e) {
  const t = e.lastIndexOf("/");
  return t >= 0 ? e.slice(t + 1) : e;
}
function Pr(e) {
  let t = 0;
  for (let n = 0; n < e.length; n += 1) t = t * 31 + e.charCodeAt(n) | 0;
  return `check-${Math.abs(t)}`;
}
function Lc(e, t) {
  return `\`\`\`${t}
${e.replace(/\`\`\`/g, "``\\`")}
\`\`\``;
}
const Xc = "data:image/png;base64,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", Cc = "data:image/png;base64,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", Oc = "data:image/png;base64,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", Sc = "data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHZpZXdCb3g9IjAgMCA3MjAgMTYwIiB3aWR0aD0iNzIwIiBoZWlnaHQ9IjE2MCI+CiAgCiAgPHRleHQgeD0iMCIgeT0iMTIwIiBjbGFzcz0iYXQtbGV0dGVyIiBmaWxsPSIjRjBFQUQ4Ij5BVDwvdGV4dD4KICA8ZyB0cmFuc2Zvcm09InRyYW5zbGF0ZSgyMzIgNTYpIj4KICAgIDxjaXJjbGUgY3g9IjUwIiBjeT0iNDgiIHI9IjMyIiBmaWxsPSJub25lIiBzdHJva2U9IiNGMEVBRDgiIHN0cm9rZS13aWR0aD0iMjIiPjwvY2lyY2xlPgogICAgPGVsbGlwc2UgY3g9IjUwIiBjeT0iNDgiIHJ4PSI0OCIgcnk9IjIwIiBmaWxsPSJub25lIiBzdHJva2U9IiNDNEZGMDAiIHN0cm9rZS13aWR0aD0iOSIgdHJhbnNmb3JtPSJyb3RhdGUoLTIyIDUwIDQ4KSI+PC9lbGxpcHNlPgogIDwvZz4KICA8dGV4dCB4PSIzNTAiIHk9IjEyMCIgY2xhc3M9ImF0LWxldHRlciIgZmlsbD0iI0YwRUFEOCI+TUVLPC90ZXh0Pgo8L3N2Zz4=";
function Nr({ size: e = 28, variant: t = "acid", className: n }) {
  return /* @__PURE__ */ a(
    "img",
    {
      src: t === "icon" ? Oc : t === "cream" ? Cc : Xc,
      alt: "Atomek",
      className: n,
      width: e,
      height: e,
      draggable: !1,
      style: { width: e, height: e }
    }
  );
}
function Hc({ className: e }) {
  return /* @__PURE__ */ a("img", { src: Sc, alt: "ATOMEK", className: e, draggable: !1 });
}
const Tr = {
  "app.agentTeam": "Agent Team",
  "app.workspace": "Workspace",
  "app.settings": "Settings",
  "app.ready": "Ready",
  "activity.aria": "Activity Bar",
  "activity.toggleSidebar": "Toggle side bar",
  "activity.explorer": "Explorer",
  "activity.search": "Search",
  "shell.resizeExplorer": "Resize Explorer",
  "shell.resizeChat": "Resize Chat",
  "shell.saveAll": "Save all",
  "shell.dismiss": "Dismiss",
  "shell.loadingEditor": "Loading editor…",
  "shell.noEditor": "No editor open",
  "shell.showAgentTeam": "Show Agent Team",
  "explorer.title": "EXPLORER",
  "explorer.noFolder": "NO FOLDER OPENED",
  "explorer.noFolderBody": "You have not yet opened a folder.",
  "explorer.openFolder": "Open Folder",
  "explorer.openFile": "Open File",
  "explorer.openRecent": "Open Recent",
  "explorer.noRecentWorkspace": "No recent local workspace yet.",
  "explorer.fsAccess": "Local files use browser-native File System Access API.",
  "explorer.browserFallback": "Browser fallback may show a file chooser label.",
  "explorer.searchFiles": "Search files",
  "explorer.openEditors": "Open Editors",
  "explorer.noOpenEditors": "No open editors",
  "explorer.noReadableFiles": "No readable text files found.",
  "explorer.recent": "Recent",
  "explorer.noRecentFolders": "No recent folders yet.",
  "search.title": "SEARCH",
  "search.placeholder": "Search files and text",
  "search.results": "Results",
  "search.help": "Type to search filenames and text in the opened workspace.",
  "search.noMatches": "No matches.",
  "search.moreMatches": "+{count} more matches",
  "tabs.atomekSettings": "Atomek Settings",
  "tabs.save": "Save",
  "tabs.toggleMarkdownPreview": "Toggle Markdown Preview",
  "tabs.toggleChat": "Toggle Chat",
  "secondary.chat": "CHAT",
  "secondary.agents": "AGENTS",
  "secondary.outputs": "OUTPUTS",
  "secondary.newChat": "New Chat",
  "secondary.pastConversations": "Past Conversations",
  "secondary.chatSettings": "Chat Settings",
  "secondary.closeChat": "Close Chat",
  "history.search": "Search all conversations...",
  "history.current": "Current",
  "history.recent": "Recent",
  "history.agent": "Agent",
  "history.deleteConversation": "Delete conversation",
  "history.noMatches": "No matches",
  "history.empty": "No conversations yet — start chatting with any agent.",
  "history.open": "open",
  "history.close": "close",
  "chat.atomekChat": "Atomek chat",
  "chat.session": "{name} session",
  "chat.newConversation": "New conversation",
  "chat.clearConversation": "Clear conversation",
  "chat.history": "Chat history",
  "chat.chats": "Chats ({count})",
  "chat.actions": "Chat actions",
  "chat.rename": "Rename",
  "chat.renamePrompt": "Rename chat",
  "chat.delete": "Delete",
  "chat.buildWith": "Build with {name}",
  "chat.emptyAtomek": "Ask about open files, request a plan, or draft an artifact.",
  "chat.emptyPod": "Chat with the selected pod agent from this same Atomek panel.",
  "chat.you": "You",
  "chat.streaming": "streaming",
  "chat.error": "error",
  "chat.copyAnswer": "Copy answer",
  "chat.saveArtifact": "Save as output artifact",
  "chat.remember": "Remember",
  "chat.previewPatch": "Preview patch",
  "chat.regenerate": "Regenerate",
  "chat.copyError": "Copy error",
  "chat.retry": "Retry",
  "chat.jumpLatest": "Jump to latest",
  "chat.target": "Target",
  "chat.context": "Context",
  "chat.noContext": "No context",
  "chat.selection": "Selection",
  "chat.activeFile": "Active file",
  "chat.openEditors": "Open editors",
  "chat.indexedProject": "Indexed project",
  "chat.index": "Index",
  "chat.stale": "stale",
  "chat.noFileContext": "No file context",
  "chat.revealContext": "Reveal context",
  "chat.dirty": "dirty",
  "chat.removeContext": "Remove context",
  "chat.explain": "Explain",
  "chat.improve": "Improve",
  "chat.edit": "Edit",
  "chat.draft": "Draft",
  "chat.generatePatch": "Generate patch for last edit request",
  "chat.askAtomek": "Ask Atomek about the open file or describe what to build...",
  "chat.askTarget": "Ask {name}…",
  "chat.addContext": "Add context",
  "chat.useActiveFile": "Use active file as context",
  "chat.useSelection": "Use current selection",
  "chat.useOpenEditors": "All open editors",
  "chat.useIndexedProject": "Project-wide retrieval (semantic + keyword)",
  "chat.cancelRecording": "Cancel recording",
  "chat.voiceInput": "Voice input",
  "chat.voiceUnsupported": "Voice input not supported in this browser",
  "chat.chooseTarget": "Choose chat target",
  "chat.stop": "Stop",
  "chat.send": "Send",
  "chat.sendMessage": "Send message",
  "status.files": "{count} files",
  "status.unsaved": "{count} unsaved",
  "status.lineColumn": "Ln {line}, Col {column}",
  "status.spaces": "Spaces: 2"
}, jc = {
  "app.agentTeam": "Equipo de agentes",
  "app.workspace": "Workspace",
  "app.settings": "Ajustes",
  "app.ready": "Listo",
  "activity.aria": "Barra de actividad",
  "activity.toggleSidebar": "Mostrar/ocultar barra lateral",
  "activity.explorer": "Explorador",
  "activity.search": "Buscar",
  "shell.resizeExplorer": "Redimensionar explorador",
  "shell.resizeChat": "Redimensionar chat",
  "shell.saveAll": "Guardar todo",
  "shell.dismiss": "Cerrar",
  "shell.loadingEditor": "Cargando editor…",
  "shell.noEditor": "Ningún editor abierto",
  "shell.showAgentTeam": "Mostrar equipo de agentes",
  "explorer.title": "EXPLORADOR",
  "explorer.noFolder": "NINGUNA CARPETA ABIERTA",
  "explorer.noFolderBody": "Todavía no has abierto una carpeta.",
  "explorer.openFolder": "Abrir carpeta",
  "explorer.openFile": "Abrir archivo",
  "explorer.openRecent": "Abrir reciente",
  "explorer.noRecentWorkspace": "Todavía no hay workspace local reciente.",
  "explorer.fsAccess": "Los archivos locales usan la API nativa File System Access del navegador.",
  "explorer.browserFallback": "El fallback del navegador puede mostrar un selector de archivos.",
  "explorer.searchFiles": "Buscar archivos",
  "explorer.openEditors": "Editores abiertos",
  "explorer.noOpenEditors": "No hay editores abiertos",
  "explorer.noReadableFiles": "No se encontraron archivos de texto legibles.",
  "explorer.recent": "Recientes",
  "explorer.noRecentFolders": "Todavía no hay carpetas recientes.",
  "search.title": "BUSCAR",
  "search.placeholder": "Buscar archivos y texto",
  "search.results": "Resultados",
  "search.help": "Escribe para buscar nombres de archivo y texto en el workspace abierto.",
  "search.noMatches": "Sin coincidencias.",
  "search.moreMatches": "+{count} coincidencias más",
  "tabs.atomekSettings": "Ajustes de Atomek",
  "tabs.save": "Guardar",
  "tabs.toggleMarkdownPreview": "Mostrar/ocultar vista previa Markdown",
  "tabs.toggleChat": "Mostrar/ocultar chat",
  "secondary.chat": "CHAT",
  "secondary.agents": "AGENTES",
  "secondary.outputs": "SALIDAS",
  "secondary.newChat": "Nuevo chat",
  "secondary.pastConversations": "Conversaciones anteriores",
  "secondary.chatSettings": "Ajustes del chat",
  "secondary.closeChat": "Cerrar chat",
  "history.search": "Buscar en todas las conversaciones...",
  "history.current": "Actual",
  "history.recent": "Recientes",
  "history.agent": "Agente",
  "history.deleteConversation": "Eliminar conversación",
  "history.noMatches": "Sin coincidencias",
  "history.empty": "Todavía no hay conversaciones — empieza a chatear con cualquier agente.",
  "history.open": "abrir",
  "history.close": "cerrar",
  "chat.atomekChat": "Chat de Atomek",
  "chat.session": "Sesión de {name}",
  "chat.newConversation": "Nueva conversación",
  "chat.clearConversation": "Limpiar conversación",
  "chat.history": "Historial de chat",
  "chat.chats": "Chats ({count})",
  "chat.actions": "Acciones del chat",
  "chat.rename": "Renombrar",
  "chat.renamePrompt": "Renombrar chat",
  "chat.delete": "Eliminar",
  "chat.buildWith": "Construir con {name}",
  "chat.emptyAtomek": "Pregunta sobre archivos abiertos, pide un plan o redacta un artefacto.",
  "chat.emptyPod": "Chatea con el agente pod seleccionado desde este mismo panel de Atomek.",
  "chat.you": "Tú",
  "chat.streaming": "transmitiendo",
  "chat.error": "error",
  "chat.copyAnswer": "Copiar respuesta",
  "chat.saveArtifact": "Guardar como artefacto de salida",
  "chat.remember": "Recordar",
  "chat.previewPatch": "Previsualizar parche",
  "chat.regenerate": "Regenerar",
  "chat.copyError": "Copiar error",
  "chat.retry": "Reintentar",
  "chat.jumpLatest": "Ir a lo último",
  "chat.target": "Objetivo",
  "chat.context": "Contexto",
  "chat.noContext": "Sin contexto",
  "chat.selection": "Selección",
  "chat.activeFile": "Archivo activo",
  "chat.openEditors": "Editores abiertos",
  "chat.indexedProject": "Proyecto indexado",
  "chat.index": "Índice",
  "chat.stale": "desactualizado",
  "chat.noFileContext": "Sin contexto de archivo",
  "chat.revealContext": "Mostrar contexto",
  "chat.dirty": "sin guardar",
  "chat.removeContext": "Quitar contexto",
  "chat.explain": "Explicar",
  "chat.improve": "Mejorar",
  "chat.edit": "Editar",
  "chat.draft": "Redactar",
  "chat.generatePatch": "Generar parche para la última petición de edición",
  "chat.askAtomek": "Pregunta a Atomek sobre el archivo abierto o describe qué construir...",
  "chat.askTarget": "Pregunta a {name}…",
  "chat.addContext": "Añadir contexto",
  "chat.useActiveFile": "Usar archivo activo como contexto",
  "chat.useSelection": "Usar selección actual",
  "chat.useOpenEditors": "Todos los editores abiertos",
  "chat.useIndexedProject": "Recuperación en todo el proyecto (semántica + keywords)",
  "chat.cancelRecording": "Cancelar grabación",
  "chat.voiceInput": "Entrada de voz",
  "chat.voiceUnsupported": "Entrada de voz no compatible en este navegador",
  "chat.chooseTarget": "Elegir objetivo de chat",
  "chat.stop": "Detener",
  "chat.send": "Enviar",
  "chat.sendMessage": "Enviar mensaje",
  "status.files": "{count} archivos",
  "status.unsaved": "{count} sin guardar",
  "status.lineColumn": "Lín {line}, Col {column}",
  "status.spaces": "Espacios: 2"
}, ja = { en: Tr, es: jc }, Ma = (e) => (e || "en").toLowerCase().split("-")[0] || "en", Mc = (e, t) => t ? e.replace(/\{(\w+)\}/g, (n, r) => String(t[r] ?? `{${r}}`)) : e, zr = gs({ locale: "en", t: (e) => Tr[e] ?? e });
function Wc({ host: e, children: t }) {
  const [n, r] = T(() => Ma(e.i18n?.locale));
  oe(() => e.i18n?.onLocaleChange((s) => r(Ma(s))), [e]);
  const i = ue(() => ({
    locale: n,
    t: (s, o) => {
      const c = e.i18n?.t(s, o);
      return c && c !== s ? c : Mc(ja[n]?.[s] ?? ja.en[s] ?? s, o);
    }
  }), [e, n]);
  return /* @__PURE__ */ a(zr.Provider, { value: i, children: t });
}
function Ke() {
  return ws(zr).t;
}
const Vc = `# Tytus Resource Fabric

Tytus Resource Fabric connects the local computer, Tytus pods, shared folders, local AI agents, app skills, channels, and global AIL routes around one mission.

## Core resources

| Resource | Job |
|---|---|
| Local computer | Real files, terminal, installed apps, browser sessions, local CLIs. |
| OpenClaw | Fast pod agent for critique, planning, and remote workflows. |
| Hermes | Deeper pod reasoning and synthesis when allocated. |
| Shared folders | Exchange layer for agents, pods, and apps. |
| Local agents | Claude, OpenCode, Codex, pi, Kimi, Gemini, Qwen, Aider, and similar CLIs. |
| App skills | Instructions/drivers for Atomek, JULI3TA, Blender, Remotion, and installed apps. |
| Channels | Telegram/Slack/Discord-style communication when configured. |

## Mission loop

1. Create a mission folder.
2. Attach local files, shared folders, resources, and current task.
3. Ask OpenClaw/Hermes for pod perspective.
4. Run local agents or app skills for local execution.
5. Save transcripts under \`runs/\` and outputs under \`outputs/\`.
6. Convert patches into previews before applying.
7. Put final handoff in \`OUTBOX.md\`.

## Rules

- No hardcoded model ids. AIL routes provide model aliases globally.
- No raw browser calls to pod/model URLs. Use Tytus host bridge.
- No arbitrary shell from model text. Use allowlisted tools.
- No blind writes. Use proposals, previews, and approvals.
`, Ic = `# OpenClaw and Hermes

OpenClaw and Hermes are the first-class Tytus pod agent brands.

## OpenClaw

Use OpenClaw for fast remote critique, planning, task splitting, channel/app workflows, and independent pod perspective.

Good tasks:

- review a repo plan
- critique a proposed patch
- extract risks from source files
- create task cards for a mission
- inspect shared-folder inputs

## Hermes

Use Hermes when allocated for deeper reasoning, synthesis, writing, and architecture review.

Good tasks:

- final plan review
- long-form report drafting
- risk synthesis
- architectural critique
- polished handoff text

## Local agents

Use local agents for work that needs the user computer: local repos, installed tools, terminals, browser sessions, and app-specific CLIs.

Recommended team:

1. OpenClaw critiques.
2. Local agent implements or tests.
3. Hermes reviews deeply when available.
4. Atomek previews and approves outputs.
`, Fc = "# Shared Folders and Mission Folders\n\nShared folders let local agents, Tytus pods, and app skills exchange ordinary files.\n\n## Folder roles\n\n| Path | Role |\n|---|---|\n| `~/Tytus/Shared` | Local same-machine drop-zone. |\n| `~/Tytus/Missions/<mission>` | Per-job shared context and audit trail. |\n| `/app/workspace` | Pod workspace. |\n| `/app/workspace/inbox` | Pod input drop-zone. |\n| `/app/workspace/out` | Pod output drop-zone. |\n\n## Mission convention\n\n- `MISSION.md`: goal and rules\n- `RESOURCES.md`: resources selected for the job\n- `TASKS.md`: task graph\n- `INBOX.md`: incoming notes/findings\n- `OUTBOX.md`: final handoff\n- `runs/`: transcripts\n- `outputs/`: generated artifacts\n- `proposals/`: patches/write proposals\n- `approvals/`: explicit approve/reject records\n\nAgents should leave evidence in files. The next agent should be able to continue from the mission folder without asking the user to repeat context.\n", Dc = `# Mission Use Cases

## Repo repair

Open a repo in Atomek, start a mission, ask OpenClaw to critique, run local OpenCode/Claude/Codex/pi for implementation, ask Hermes for final review, then apply approved patches.

## Document package

Drop notes and PDFs into a mission folder. Ask OpenClaw to extract claims, Hermes to synthesize, and a local agent to write markdown. Review in Atomek before export.

## Creative production

Use shared folders for references, lyrics, images, audio, and render assets. Coordinate JULI3TA, Blender, Remotion, and local agents through one mission folder.

## Research watch

Use a pod or AIL route for source gathering, local agent for cleanup, and \`OUTBOX.md\` for the final concise report.

## App automation

Install an app skill, refresh capabilities, attach the mission folder, launch the app or local bridge through Tytus host integration, and keep artifacts in \`outputs/\`.
`, Rc = `# Agentic App Skills

App skills are instructions and drivers that tell agents how to use a Tytus app or local app safely.

Examples:

- Atomek inspect workspace
- Atomek patch preview
- Atomek local agent job
- JULI3TA music generation/restyle workflow
- Blender MCP scene generation
- Remotion render recipes

## Skill rules

- Show real installed skills only.
- If a tool is missing, show setup-needed instead of fake support.
- Route local app control through Tytus host/tray bridges.
- Keep source assets and generated outputs in the mission folder.
- Use previews and approvals for generated edits.
`, Ec = [
  {
    id: "resource-fabric",
    title: "Tytus Resource Fabric",
    summary: "How local computer, pods, shared folders, local agents, apps, channels, and AIL routes work together.",
    fileName: "Tytus-Resource-Fabric.md",
    body: Vc,
    tags: ["Tytus", "mission", "resources"]
  },
  {
    id: "openclaw-hermes",
    title: "OpenClaw + Hermes",
    summary: "When to use OpenClaw, Hermes, and local agents in one team.",
    fileName: "OpenClaw-Hermes-Agents.md",
    body: Ic,
    tags: ["OpenClaw", "Hermes", "agents"]
  },
  {
    id: "shared-folders",
    title: "Shared folders",
    summary: "Mission folders, INBOX/OUTBOX, pod workspaces, and agent handoff conventions.",
    fileName: "Shared-Folders.md",
    body: Fc,
    tags: ["shared", "files", "handoff"]
  },
  {
    id: "mission-use-cases",
    title: "Mission use cases",
    summary: "Repo repair, documents, creative production, research watch, and app automation.",
    fileName: "Mission-Use-Cases.md",
    body: Dc,
    tags: ["use cases", "workflow"]
  },
  {
    id: "agentic-app-skills",
    title: "Agentic app skills",
    summary: "How Tytus apps expose skills and how Atomek uses them safely.",
    fileName: "Agentic-App-Skills.md",
    body: Rc,
    tags: ["skills", "apps"]
  }
], qc = "tytus-atomek-handles", yt = "handles", Yc = 1, Ar = () => new Promise((e, t) => {
  const n = indexedDB.open(qc, Yc);
  n.onupgradeneeded = () => {
    const r = n.result;
    r.objectStoreNames.contains(yt) || r.createObjectStore(yt);
  }, n.onsuccess = () => e(n.result), n.onerror = () => t(n.error ?? new Error("Failed to open Atomek handle store"));
});
async function Wa(e, t) {
  const n = await Ar();
  await new Promise((r, i) => {
    const s = n.transaction(yt, "readwrite");
    s.objectStore(yt).put(t, e), s.oncomplete = () => r(), s.onerror = () => i(s.error ?? new Error("Failed to persist file handle"));
  }), n.close();
}
async function On(e) {
  const t = await Ar(), n = await new Promise((r, i) => {
    const o = t.transaction(yt, "readonly").objectStore(yt).get(e);
    o.onsuccess = () => r(o.result ?? null), o.onerror = () => i(o.error ?? new Error("Failed to read file handle"));
  });
  return t.close(), n;
}
const Jc = ks(() => import("./WorkbenchMonacoEditor-DeMocMpz.js").then((e) => e.W).then((e) => ({ default: e.WorkbenchMonacoEditor }))), Zc = {
  id: "welcome",
  name: "Agent Team",
  path: "Agent Team",
  language: "text",
  content: "",
  dirty: !1,
  source: "sample"
}, Lr = "tytus.workspace.recent", Xr = "tytus.workspace.layout", Cr = "tytus.atomek.session.v2", Or = "tytus.atomek.chatAiSettings", Sn = "atomek:default", Sr = "tytus.atomek.currentMission", Yn = "tytus.atomek.currentMissionChanged", Va = "0.4.32", en = {
  gatewayPreference: "auto",
  model: "",
  embeddingModel: ""
}, Kc = 48, Rt = {
  sharedFiles: "filemanager",
  podInspector: "pod-inspector",
  channels: "channels",
  settings: "settings"
};
function Uc(e) {
  return e === "search" || e === "computer" ? e : "explorer";
}
function Bc(e) {
  return e === "terminal" ? "terminal" : "output";
}
function Et(e, t, n) {
  return Math.round(Math.max(t, Math.min(n, e)));
}
function Hn(e) {
  const t = Math.max(e || 1400, 760), n = Math.max(0, t - Kc), r = n < 1180, i = r ? 200 : 240, s = r ? 280 : 300, o = r ? 360 : 420, c = Math.max(i, Math.min(r ? 340 : 420, Math.floor(n * 0.28))), m = Math.min(300, c), b = Math.floor(n * (r ? 0.5 : 0.62)), p = n - m - o, v = Math.max(s, Math.min(r ? 1e3 : 1400, b, p));
  return { primaryMin: i, primaryMax: c, secondaryMin: s, secondaryMax: v };
}
function Gc(e) {
  const t = [], n = /```([^\n`]*)\n?([\s\S]*?)```/g;
  let r = 0, i = 0, s;
  for (; (s = n.exec(e)) !== null; ) {
    if (s.index > r) {
      const c = e.slice(r, s.index);
      c.trim() && t.push({ type: "markdown", body: c, key: `md-${i}` });
    }
    t.push({
      type: "code",
      language: s[1]?.trim() || "text",
      body: s[2] ?? "",
      key: `code-${i}`
    }), r = s.index + s[0].length, i += 1;
  }
  const o = e.slice(r);
  return (o.trim() || t.length === 0) && t.push({ type: "markdown", body: o, key: `md-${i}` }), t;
}
async function gt(e) {
  if (!e) return !1;
  try {
    return await navigator.clipboard?.writeText(e), !0;
  } catch {
    const t = document.createElement("textarea");
    t.value = e, t.setAttribute("readonly", "true"), t.style.position = "fixed", t.style.opacity = "0", t.style.pointerEvents = "none", document.body.appendChild(t), t.select();
    const n = document.execCommand("copy");
    return document.body.removeChild(t), n;
  }
}
async function Qc() {
  const e = window;
  return typeof e.showDirectoryPicker != "function" ? null : e.showDirectoryPicker({ mode: "readwrite" });
}
async function Jn(e, t, n) {
  const r = e.getFileHandle;
  if (!r) throw new Error("Selected mission folder is read-only in this browser context");
  const i = await r.call(e, t, { create: !0 });
  if (!i.createWritable) throw new Error(`Cannot write ${t}; File System Access write handle unavailable`);
  const s = await i.createWritable();
  await s.write(n), await s.close();
}
async function pt(e, t) {
  const n = e.getDirectoryHandle;
  if (!n) throw new Error("Selected mission folder cannot create subfolders in this browser context");
  return n.call(e, t, { create: !0 });
}
async function $c(e, t, n) {
  const r = t.split("/").filter(Boolean);
  if (r.length === 0) return;
  let i = e;
  for (const s of r.slice(0, -1))
    i = await pt(i, s);
  await Jn(i, r[r.length - 1], n);
}
function Hr(e) {
  const t = e.reduce((n, r) => (n[r.kind] = (n[r.kind] ?? 0) + 1, n), {});
  return Object.entries(t).map(([n, r]) => `${r} ${n}`).join(" · ") || "no resources";
}
function nt(e, t) {
  const n = e.metadata?.[t];
  return typeof n == "string" ? n : typeof n == "number" ? String(n) : "";
}
function Zn(e) {
  const t = nt(e, "podId");
  return t || (e.id.match(/(?:pod-agent|ail-route)\.([^.]+)/)?.[1] ?? "");
}
function _c(e) {
  return nt(e, "routeId") || nt(e, "route_id") || null;
}
function wt(e) {
  const t = [
    nt(e, "agentFamily"),
    nt(e, "agentType"),
    nt(e, "internalAgentType"),
    nt(e, "brand"),
    e.label
  ].join(" ").toLowerCase();
  return e.kind === "ail-route" || /(^|[^a-z0-9])ail([^a-z0-9]|$)/.test(t) ? "ail" : t.includes("hermes") ? "hermes" : t.includes("openclaw") || t.includes("nemoclaw") ? "openclaw" : null;
}
function je(e) {
  const t = wt(e), n = Zn(e);
  return e.kind === "pod-agent" && t === "openclaw" ? `OpenClaw agent${n ? ` pod ${n}` : ""}` : e.kind === "pod-agent" && t === "hermes" ? `Hermes agent${n ? ` pod ${n}` : ""}` : e.kind === "ail-route" ? `AIL gateway${n ? ` ${n}` : ""}` : e.label.replace(/\bNemoClaw\b/g, "OpenClaw").replace(/\bnemoclaw\b/gi, "OpenClaw");
}
function oa(e) {
  if (e.kind === "pod-agent") {
    const t = wt(e), n = nt(e, "units");
    return `${t === "openclaw" ? "OpenClaw pod agent" : t === "hermes" ? "Hermes reasoning agent" : "Tytus pod agent"}${n ? ` · ${n} unit${n === "1" ? "" : "s"}` : ""} · ${e.trustTier}`;
  }
  return e.kind === "local-cli" ? `Local CLI · ${e.capabilities.slice(0, 3).join(", ") || "tool launch"}` : e.kind === "shared-folder" ? `Shared folder · ${e.sandbox}` : e.kind === "app-skill" ? `App skill · ${e.capabilities.slice(0, 3).join(", ") || "skill instructions"}` : e.kind === "ail-route" ? `Remote AIL route · ${e.capabilities.slice(0, 3).join(", ") || "text-gen"}` : `${e.kind} · ${e.trustTier}`;
}
function el(e) {
  const t = e?.resources ?? [], n = (c) => c.status === "ready" || c.status === "available", r = t.filter((c) => c.kind === "pod-agent" && wt(c) === "openclaw"), i = t.filter((c) => c.kind === "pod-agent" && wt(c) === "hermes"), s = t.filter((c) => c.kind === "local-cli" && n(c)), o = t.filter((c) => c.kind === "shared-folder" && n(c));
  return [
    { label: "OpenClaw", value: r.length, detail: "fast pod agents for critique, planning, channel/app workflows", status: r.some(n) ? "ready" : "not allocated" },
    { label: "Hermes", value: i.length, detail: "heavier pod agent family when allocated", status: i.some(n) ? "ready" : "available when installed" },
    { label: "Local agents", value: s.length, detail: "Claude, OpenCode, Codex, pi, Kimi through Tytus tray", status: s.length ? "ready" : "missing" },
    { label: "Shared folders", value: o.length, detail: "mission context and handoff fabric for the whole team", status: o.length ? "ready" : "needs setup" }
  ];
}
function tl(e) {
  const t = e?.resources ?? [], n = (r) => t.some((i) => i.kind === r && (i.status === "ready" || i.status === "available"));
  return [
    { label: "Local computer", detail: n("local-cli") ? "Local CLIs, files, terminal, and installed apps are reachable through Tytus tray." : "Waiting for local tools from Tytus tray.", status: n("local-cli") ? "ready" : "needs setup" },
    { label: "Shared folders", detail: n("shared-folder") ? "Files, transcripts, patches, and artifacts can move between local agents and pods." : "Bind a shared folder to create the exchange layer.", status: n("shared-folder") ? "ready" : "needs setup" },
    { label: "Tytus pods", detail: n("pod-agent") ? "OpenClaw/Hermes pods can pick up context and return remote work products." : "No pod agent is ready yet.", status: n("pod-agent") ? "ready" : "needs setup" },
    { label: "Local apps", detail: n("app-skill") ? "App skills expose JULI3TA, Blender, Remotion, and other tools as mission capabilities." : "App skills appear when installed/configured.", status: n("app-skill") ? "ready" : "optional" }
  ];
}
const Ct = [
  {
    id: "repo-repair",
    label: "Repo Repair",
    summary: "Local implementer plus independent reviewer, with all transcripts in the mission folder.",
    bestFor: "code fixes, docs, release cleanup"
  },
  {
    id: "pod-local",
    label: "OpenClaw + Local",
    summary: "OpenClaw/Hermes pod perspective plus local Claude/OpenCode/Codex/pi execution.",
    bestFor: "cross-agent critique, planning, distributed work"
  },
  {
    id: "creative-production",
    label: "Creative Production",
    summary: "App skills and local/pod agents share source assets, scripts, outputs, and approvals.",
    bestFor: "JULI3TA, Blender, Remotion, media pipelines"
  },
  {
    id: "research-watch",
    label: "Research Watch",
    summary: "Remote/pod research, local synthesis, shared-folder handoff, optional channel updates.",
    bestFor: "monitoring, summaries, recurring intelligence"
  }
];
function jr(e) {
  return e.status === "ready" || e.status === "available" || e.status === "degraded";
}
function nl(e) {
  return [
    e.id,
    e.label,
    e.kind,
    je(e),
    oa(e),
    e.capabilities.join(" "),
    Object.values(e.metadata ?? {}).join(" ")
  ].join(" ").toLowerCase();
}
function ot(e, t) {
  return e.find((n) => jr(n) && t(n)) ?? null;
}
function Qe(e, t, n) {
  return ot(e, (r) => {
    if (r.kind !== t) return !1;
    const i = nl(r);
    return n.some((s) => {
      const o = s.trim().toLowerCase();
      if (!o) return !1;
      if (o.length <= 4) {
        const c = o.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
        return new RegExp(`(^|[^a-z0-9])${c}([^a-z0-9]|$)`).test(i);
      }
      return i.includes(o);
    });
  });
}
function At(e, t, n, r, i) {
  return {
    role: e,
    label: t,
    purpose: n,
    resourceId: r?.id ?? `missing:${e}`,
    resourceLabel: r ? je(r) : i,
    status: r?.status ?? "needs-setup",
    trustTier: r?.trustTier ?? "not-available",
    sandbox: r?.sandbox ?? "none"
  };
}
function kt(e, t = "repo-repair") {
  const n = Ct.find((I) => I.id === t) ?? Ct[0], r = e?.resources ?? [], i = ot(r, (I) => I.kind === "pod-agent" && wt(I) === "openclaw"), s = ot(r, (I) => I.kind === "pod-agent" && wt(I) === "hermes"), o = ot(r, (I) => I.kind === "ail-route"), c = ot(r, (I) => I.kind === "shared-folder"), m = Qe(r, "local-cli", ["claude"]), b = Qe(r, "local-cli", ["opencode", "open code"]), p = Qe(r, "local-cli", ["codex"]), v = Qe(r, "local-cli", ["pi"]), g = Qe(r, "local-cli", ["kimi"]), S = ot(r, (I) => I.kind === "local-cli"), M = Qe(r, "app-skill", ["juli3ta", "music", "song"]), E = Qe(r, "app-skill", ["blender", "3d"]), J = Qe(r, "app-skill", ["remotion", "video", "render"]), F = ot(r, (I) => I.kind === "app-skill"), ae = t === "pod-local" ? i ?? s ?? o ?? m ?? S : t === "creative-production" ? s ?? o ?? m ?? S : t === "research-watch" ? i ?? o ?? s ?? m ?? S : m ?? b ?? o ?? S, me = t === "creative-production" ? M ?? E ?? J ?? F ?? b ?? m ?? S : b ?? m ?? p ?? S, W = t === "pod-local" ? p ?? v ?? g ?? i ?? m ?? S : t === "research-watch" ? v ?? g ?? p ?? i ?? S : p ?? v ?? i ?? g ?? S, Z = t === "creative-production" ? M ?? E ?? J ?? F : t === "research-watch" ? Qe(r, "app-skill", ["browser", "channel", "telegram"]) ?? F : F, K = [
    At("planner", "Planner", "Break goal into tasks, risks, and required context.", ae, "No planner agent ready"),
    At("implementer", "Implementer", "Execute local/app work and return transcript or patch proposal.", me, "No local/app implementer ready"),
    At("reviewer", "Reviewer", "Independent critique before approval or handoff.", W, "No reviewer agent ready"),
    At("team-desk", "Team Desk", "Shared mission folder for transcripts, outputs, proposals, and handoff.", c, "Mission folder only until shared folder is bound")
  ];
  (t === "creative-production" || Z) && K.push(At("app-tool", "App Tool", "Drive installed local app skill through mission context.", Z, "No configured app skill"));
  const V = K.filter((I) => I.status === "ready" || I.status === "available" || I.status === "degraded").length, z = V >= K.length - 1 ? "ready" : V >= 2 ? "partial" : "needs-setup";
  return { ...n, readiness: z, assignments: K };
}
function lt(e, t, n) {
  if (n && Ct.some((i) => i.id === n)) return n;
  const r = e.toLowerCase();
  return /(music|song|audio|video|render|blender|remotion|juli3ta|creative)/.test(r) ? "creative-production" : /(research|watch|monitor|news|summar|scan)/.test(r) ? "research-watch" : (t?.resources ?? []).some((i) => i.kind === "pod-agent" && jr(i)) ? "pod-local" : "repo-repair";
}
function al(e) {
  return Ct.map((t) => kt(e, t.id));
}
function qt(e) {
  return `${e.label}: ${e.resourceLabel} (${e.status})`;
}
function rl(e) {
  return e.toLowerCase().replace(/[^a-z0-9]+/g, "-").replace(/^-+|-+$/g, "").slice(0, 40) || "mission";
}
function ft() {
  return (/* @__PURE__ */ new Date()).toISOString();
}
function Ia(e) {
  return e.finishedAt ?? e.startedAt ?? "";
}
function Lt(e) {
  const t = "source" in e ? e : {
    missionId: e.missionId,
    title: e.title,
    goal: e.goal,
    rootPath: e.rootPath,
    name: e.rootPath.split("/").pop() || e.missionId,
    source: "tray",
    teamPresetId: void 0
  };
  try {
    localStorage.setItem(Sr, JSON.stringify(t)), window.dispatchEvent(new CustomEvent(Yn, { detail: t }));
  } catch {
  }
}
function sl() {
  try {
    const e = localStorage.getItem(Sr);
    if (!e) return null;
    const t = JSON.parse(e);
    return !t.missionId || !t.title ? null : {
      missionId: t.missionId,
      title: t.title,
      goal: t.goal ?? "",
      rootPath: t.rootPath,
      name: t.name ?? t.rootPath?.split("/").pop() ?? t.missionId,
      source: t.source === "browser" ? "browser" : "tray",
      teamPresetId: Ct.some((n) => n.id === t.teamPresetId) ? t.teamPresetId : void 0
    };
  } catch {
    return null;
  }
}
function il(e) {
  return {
    missionId: e.missionId,
    title: e.title,
    goal: e.goal,
    rootPath: e.rootPath,
    name: e.rootPath.split("/").pop() || e.missionId,
    source: "tray",
    teamPresetId: void 0
  };
}
function Ot(e, t, n) {
  const r = e.trim() || "Coordinate a Tytus mission.", i = kt(t, lt(r, t, n)), s = (v) => i.assignments.find((g) => g.role === v), o = s("planner"), c = s("implementer"), m = s("reviewer"), b = s("team-desk"), p = s("app-tool");
  return [
    {
      id: "task-scope",
      title: "Scope mission and context",
      prompt: `Planner role: turn this goal into an executable mission plan, list required files/assets, and define approval gates. Goal: ${r}`,
      resourceHint: o ? qt(o) : "Planner agent",
      role: "planner",
      assignedResourceLabel: o?.resourceLabel ?? "Planner agent",
      status: "ready",
      expectedOutputs: ["PLAN.md", "risk list", "resource choices"]
    },
    {
      id: "task-execute",
      title: "Execute or produce artifact",
      prompt: `Implementer role: use the mission folder, selected files, and shared/team desk context to execute the smallest safe step. Return transcript, artifact, or patch proposal only. Goal: ${r}`,
      resourceHint: c ? qt(c) : "Local/app implementer",
      role: "implementer",
      assignedResourceLabel: c?.resourceLabel ?? "Local/app implementer",
      status: "waiting",
      expectedOutputs: ["transcript", "artifact", "patch proposal"]
    },
    p ? {
      id: "task-app-tool",
      title: "Drive app skill",
      prompt: `App-tool role: use the relevant app skill only through configured Tytus/app instructions. Save source assets and outputs into the mission/team desk. Goal: ${r}`,
      resourceHint: qt(p),
      role: "app-tool",
      assignedResourceLabel: p.resourceLabel,
      status: p.status === "ready" || p.status === "available" ? "waiting" : "needs-approval",
      expectedOutputs: ["app output", "asset path", "usage notes"]
    } : null,
    {
      id: "task-handoff",
      title: "Review and hand off",
      prompt: `Reviewer role: independently review the outputs, list approval decisions, and prepare a handoff that another agent can continue from. Goal: ${r}`,
      resourceHint: m ? `${qt(m)} · ${b?.resourceLabel ?? "mission folder"}` : "Reviewer + mission folder",
      role: "reviewer",
      assignedResourceLabel: m?.resourceLabel ?? "Reviewer agent",
      status: "waiting",
      expectedOutputs: ["REVIEW.md", "HANDOFF.md", "approval list"]
    }
  ].filter(Boolean);
}
function Mr(e) {
  return [
    "# Mission tasks",
    "",
    ...e.map((t, n) => [
      `## ${n + 1}. ${t.title}`,
      "",
      `- ID: \`${t.id}\``,
      `- Status: ${t.status}`,
      `- Role: ${t.role}`,
      `- Assigned resource: ${t.assignedResourceLabel}`,
      `- Resource hint: ${t.resourceHint}`,
      `- Expected outputs: ${t.expectedOutputs.join(", ")}`,
      "",
      t.prompt,
      ""
    ].join(`
`))
  ].join(`
`);
}
function Wr(e) {
  return [
    `# Handoff — ${e.title}`,
    "",
    `- Mission ID: \`${e.missionId}\``,
    `- Root: \`${e.rootPath ?? e.name}\``,
    `- Updated: ${(/* @__PURE__ */ new Date()).toISOString()}`,
    "",
    "## What changed",
    "",
    "- TBD",
    "",
    "## Decisions",
    "",
    "- TBD",
    "",
    "## Open approvals",
    "",
    "- No direct writes without Atomek preview/approval.",
    "",
    "## Next owner",
    "",
    "- Pick the next OpenClaw, Hermes, local-agent, shared-folder, or app-skill resource from Atomek.",
    ""
  ].join(`
`);
}
function tn(e, t, n, r, i, s) {
  const o = kt(t, lt(i || e.goal, t, s ?? e.teamPresetId));
  return [
    `# ${e.title}`,
    "",
    `- Mission ID: \`${e.missionId}\``,
    `- Updated: ${(/* @__PURE__ */ new Date()).toISOString()}`,
    `- Folder: ${e.rootPath ?? e.name}`,
    "",
    "## Goal",
    "",
    e.goal || "(no goal set)",
    "",
    "## Team preset",
    "",
    `- Preset: ${o.label} (${o.readiness})`,
    `- Best for: ${o.bestFor}`,
    `- Summary: ${o.summary}`,
    "",
    ...o.assignments.map((c) => `- ${c.label}: ${c.resourceLabel} — ${c.purpose} [${c.status}, ${c.trustTier}, ${c.sandbox}]`),
    "",
    "## Current Atomek context",
    "",
    n ? `- Active file: \`${n.path}\` (${n.language}, ${n.content.length} chars${n.dirty ? ", dirty" : ""})` : "- Active file: none",
    `- Open editors: ${r.length}`,
    "",
    "## Current task",
    "",
    i || "(no task prompt set)",
    "",
    "## Resource graph",
    "",
    t ? `- ${Hr(t.resources)}` : "- not loaded",
    ...t?.warnings?.length ? t.warnings.map((c) => `- Warning: ${c.code} — ${c.message}`) : [],
    "",
    "## Rules",
    "",
    "- Mission folder is the shared source of truth.",
    "- Shared folder / Team Desk is the exchange layer between local computer and Tytus pods when available.",
    "- Agents must not write project files directly.",
    "- Proposed edits must be returned as unified diffs or fenced replacement blocks.",
    "- Atomek previews and approves edits before applying.",
    "- Secrets are never requested or copied into mission context."
  ].join(`
`);
}
function nn(e) {
  return e ? [
    "# Resources",
    "",
    `Generated: ${e.generatedAt}`,
    "",
    ...e.resources.map((t) => [
      `## ${je(t)}`,
      "",
      `- ID: \`${t.id}\``,
      `- Kind: ${t.kind}`,
      `- Status: ${t.status}${t.reason ? ` — ${t.reason}` : ""}`,
      `- Trust: ${t.trustTier}`,
      `- Sandbox: ${t.sandbox}`,
      `- Capabilities: ${t.capabilities.join(", ") || "none"}`,
      t.allowedRoots.length ? `- Allowed roots: ${t.allowedRoots.map((n) => `\`${n}\``).join(", ")}` : "- Allowed roots: none",
      ""
    ].join(`
`)),
    e.warnings.length ? `## Warnings
` : "",
    ...e.warnings.map((t) => `- ${t.code}: ${t.message}${t.resourceId ? ` (${t.resourceId})` : ""}`),
    ""
  ].join(`
`) : `# Resources

Resource graph not loaded yet.
`;
}
function Vr(e, t, n, r) {
  const i = lt(n || e.goal, t, r ?? e.teamPresetId), s = kt(t, i), o = Ot(n || e.goal, t, i);
  return JSON.stringify({
    schemaVersion: 1,
    missionId: e.missionId,
    title: e.title,
    goal: e.goal,
    createdAt: (/* @__PURE__ */ new Date()).toISOString(),
    updatedAt: (/* @__PURE__ */ new Date()).toISOString(),
    status: "active",
    rootPath: e.rootPath ?? e.name,
    teamPreset: {
      id: s.id,
      label: s.label,
      readiness: s.readiness,
      summary: s.summary,
      bestFor: s.bestFor
    },
    team: s.assignments.map((c) => ({
      role: c.role,
      label: c.label,
      purpose: c.purpose,
      resourceId: c.resourceId,
      resourceLabel: c.resourceLabel,
      status: c.status,
      trustTier: c.trustTier,
      sandbox: c.sandbox
    })),
    storage: {
      missionFolder: e.rootPath ?? e.name,
      teamDesk: s.assignments.find((c) => c.role === "team-desk")?.resourceLabel ?? "mission folder",
      paths: {
        runs: "runs/",
        outputs: "outputs/",
        proposals: "proposals/",
        approvals: "approvals/",
        inbox: "INBOX.md",
        outbox: "OUTBOX.md"
      }
    },
    resources: (t?.resources ?? []).filter((c) => c.status === "ready").map((c) => ({
      resourceId: c.id,
      pinnedLabel: je(c),
      pinnedKind: c.kind,
      pinnedCapabilities: c.capabilities,
      visibility: {
        allowedRoots: c.allowedRoots,
        sandbox: c.sandbox,
        trustTier: c.trustTier
      }
    })),
    permissions: {
      fileWrite: "preview-only",
      shellExec: "allowlist-with-approval",
      netEgress: "resource-default",
      secretRead: "never"
    },
    secretsPolicy: {
      deniedGlobs: ["**/.env", "**/.env.*", "**/.ssh/**", "**/*_key*", "**/*secret*", "**/*token*", "**/id_rsa", "**/id_ed25519"],
      deniedPatterns: ["OPENAI_API_KEY\\\\s*=", "sk-[A-Za-z0-9_-]{20,}", "ANTHROPIC_API_KEY\\\\s*="]
    },
    budget: { maxRuntimeMinutes: 30, maxArtifactMb: 25 },
    tasks: o.map((c, m) => ({
      id: c.id,
      title: c.title,
      prompt: c.prompt,
      role: c.role,
      assignedResourceLabel: c.assignedResourceLabel,
      status: m === 0 ? "ready" : "waiting",
      selectedResourceHint: c.resourceHint,
      dependsOn: m === 0 ? [] : [o[m - 1].id],
      expectedOutputs: c.expectedOutputs,
      approvalGateIds: ["file-write-preview"]
    }))
  }, null, 2);
}
function ol({ host: e }) {
  const t = Ke(), n = Se(null), [r, i] = T(0), s = ue(() => Jl(), []), o = ue(() => Dl(), []), c = ue(() => Fl(Il(), o.recent), [o.recent]), [m, b] = T(() => Uc(o.activity) || "explorer"), [p, v] = T(s.primaryVisible), [g, S] = T(s.primaryWidth), [M, E] = T(o.secondaryTab ?? "chat"), [J, F] = T(s.secondaryVisible), [ae, me] = T(s.secondaryWidth), [W, Z] = T(!!o.bottomPanelVisible), [K, V] = T(() => Bc(o.bottomPanelTab)), [z, I] = T(s.markdownPreviewVisible), [ge, ee] = T(!!o.welcomeClosed), [fe, Te] = T(o.folder ? { name: o.folder.name, files: [] } : null), [He, xe] = T(o.folder?.handleKey ?? null), [f, q] = T(() => Rl(o.files)), [G, k] = T(() => (o.openEditorIds ?? []).filter((l) => (o.files ?? []).some((h) => h.id === l))), [O, x] = T(() => o.activeFileId && (o.files ?? []).some((l) => l.id === o.activeFileId) ? o.activeFileId : null), [L, R] = T(o.query ?? ""), [X, re] = T({ lineNumber: 1, column: 1 }), [he, ve] = T(null), [be, Ue] = T({}), [ke, Be] = T(Ui), [at, Pe] = T([]), [ze, Ye] = T([]), [te, U] = T(null), [ce, se] = T(o.chatInput ?? ""), [Ae, Me] = T(!1), [Ge, d] = T(""), [w, A] = T([]), [C, H] = T(null), [D, B] = T(null), [ie, Xe] = T(!1), [Le, le] = T(() => Zl()), [$, _] = T([Ze]), [Ne, We] = T(() => gi(Sn)), [Ce, ye] = T(null), [Ve, un] = T(null), [hn, mn] = T(""), [fn, pn] = T(""), [Ht, jt] = T(""), [Mt, la] = T("failed"), [Jr, da] = T(null), [Zr, j] = T("Ready"), [xt, Kr] = T(() => c), Ie = G.map((l) => f.find((h) => h.id === l)).filter(Boolean), Y = O ? f.find((l) => l.id === O) ?? null : null, ua = ue(() => Ki({ files: f, openEditorIds: G, activeFileId: O, versions: be, activeSelection: he }), [O, he, be, f, G]), ha = ue(() => ({
    scope: ke,
    removedAttachmentIds: at,
    selectedFileIds: []
  }), [ke, at]), Pt = ue(() => so(ua, f, ha), [ha, ua, f]), _e = Xo(f, { autoRefresh: !0, includeDirty: !0 }), ma = ue(() => ze.map((l) => ({
    id: l.id,
    kind: "index-hit",
    label: l.label,
    path: l.path,
    fileId: l.fileId,
    range: l.range,
    dirty: l.dirty,
    includeBody: !0,
    removable: !0,
    implicit: !1,
    score: l.score,
    keywordScore: l.keywordScore,
    vectorScore: l.vectorScore,
    snippet: l.snippet
  })), [ze]), Ur = ue(() => [...Pt.attachments, ...ma], [Pt.attachments, ma]), fa = ue(() => $.find((l) => l.id === Ne) ?? Ze, [$, Ne]), ne = Yi({ host: e, requestContext: Pt.parts, chatSettings: Le, selectedTarget: fa, setStatus: j }), rt = ue(
    () => [...ne.artifacts, ...w].sort((l, h) => h.createdAt - l.createdAt),
    [ne.artifacts, w]
  ), Wt = ie && !Y, bn = !Y && !Wt && !ge, Oe = ue(() => f.filter((l) => l.dirty), [f]), Br = ue(() => {
    const l = L.trim().toLowerCase();
    return l ? f.filter((h) => h.path.toLowerCase().includes(l)) : f;
  }, [f, L]);
  oe(() => {
    Ue((l) => {
      const h = {};
      for (const y of f) h[y.id] = l[y.id] ?? 1;
      return h;
    });
  }, [f]), oe(() => {
    Pe([]), Ye([]);
  }, [O, ke, G]), oe(() => {
    Ye([]);
  }, [f]), oe(() => {
    let l = !0;
    const h = async () => {
      const P = await Xi(e);
      l && (_(P), We((Q) => {
        const pe = P.some((st) => st.id === Q) ? Q : Ze.id;
        return pe !== Q && Na(Sn, pe), pe;
      }));
    };
    h();
    const y = e.daemon.onStateChange(() => {
      h();
    });
    return () => {
      l = !1, y?.();
    };
  }, [e]);
  const Gr = N((l) => {
    const h = $.find((y) => y.id === l) ?? Ze;
    We(h.id), Na(Sn, h.id), j(`Chat target: ${h.label}`);
  }, [$, j]), Qr = N((l) => {
    if (l.kind === "index-hit") {
      Ye((h) => h.filter((y) => y.id !== l.id)), j(`Removed project context: ${l.label}`);
      return;
    }
    Pe((h) => h.includes(l.id) ? h : [...h, l.id]), j(`Removed chat context: ${l.label}`);
  }, []), Vt = N(() => {
    Xe(!0), x(null), ee(!0), j("Atomek settings opened");
  }, []), pa = N(() => {
    Xe(!1), x((l) => l ?? G.at(-1) ?? null), G.length === 0 && ee(!1), j("Atomek settings closed");
  }, [G]), dt = N((l) => {
    Ue((h) => ({ ...h, [l]: (h[l] ?? 1) + 1 }));
  }, []), $r = N((l) => {
    l.preventDefault(), l.currentTarget.setPointerCapture?.(l.pointerId);
    const h = Hn(r), y = l.clientX, P = ae, Q = (st) => {
      const Tn = P + (y - st.clientX);
      me(Et(Tn, h.secondaryMin, h.secondaryMax));
    }, pe = () => {
      window.removeEventListener("pointermove", Q), window.removeEventListener("pointerup", pe);
    };
    window.addEventListener("pointermove", Q), window.addEventListener("pointerup", pe);
  }, [ae, r]), _r = N((l) => {
    l.preventDefault(), l.currentTarget.setPointerCapture?.(l.pointerId);
    const h = Hn(r), y = l.clientX, P = g, Q = (st) => {
      const Tn = P + (st.clientX - y);
      S(Et(Tn, h.primaryMin, h.primaryMax));
    }, pe = () => {
      window.removeEventListener("pointermove", Q), window.removeEventListener("pointerup", pe);
    };
    window.addEventListener("pointermove", Q), window.addEventListener("pointerup", pe);
  }, [g, r]), et = N((l) => {
    Kr((h) => {
      const y = [l, ...h.filter((P) => P.path !== l.path)].slice(0, 10);
      return localStorage.setItem(Lr, JSON.stringify(y)), y;
    });
  }, []), we = N((l, h) => {
    k((y) => y.includes(l.id) ? y : [...y, l.id]), x(l.id), ee(!1), da(h ?? null), re({ lineNumber: h ?? 1, column: 1 });
  }, []), es = N((l) => {
    const h = {
      id: `atomek-doc:${l.id}`,
      name: l.fileName,
      path: `Atomek Docs/${l.fileName}`,
      language: "markdown",
      content: l.body,
      dirty: !1,
      source: "sample"
    };
    q((y) => Jt(y, [h])), we(h), E("chat"), F(!0), I(!0), j(`Opened Atomek guide: ${l.title}`);
  }, [we]), ts = N((l) => {
    if (!l.fileId) return;
    const h = f.find((y) => y.id === l.fileId);
    h && (we(h, l.range?.startLineNumber ?? 1), j(`Revealed context: ${l.label}`));
  }, [f, we]), Nt = N(async () => {
    if (jn(Oe, "open new files"))
      try {
        const l = await fi();
        if (l.length === 0) return;
        q((h) => Jt(h, l)), await Promise.all(l.map(async (h) => {
          const y = h.handle ? `file:${h.path}` : void 0;
          h.handle && y && await Wa(y, h.handle), et({ name: h.name, path: h.path, kind: "file", handleKey: y, at: Date.now() });
        })), we(l[0]), j(`Opened ${l.length} local file${l.length === 1 ? "" : "s"}`);
      } catch (l) {
        l.name !== "AbortError" && j(`Open file failed: ${l.message}`);
      }
  }, [Oe, we, et]), vn = N(async () => {
    if (jn(Oe, "open another folder"))
      try {
        const l = await pi(), h = l.handle ? `folder:${l.name}:${Date.now()}` : null;
        l.handle && h && await Wa(h, l.handle), xe(h), Te(l), q(l.files), k([]), x(null), ee(!1), et({ name: l.name, path: l.name, kind: "folder", handleKey: h ?? void 0, at: Date.now() }), j(`${l.handle ? "Opened local folder" : "Opened browser fallback folder"} ${l.name} (${l.files.length} text files indexed)`);
      } catch (l) {
        l.name !== "AbortError" && j(`Open folder failed: ${l.message}`);
      }
  }, [Oe, et]), ns = N((l) => {
    if (!O) return;
    let h = !1;
    q((y) => y.map((P) => P.id !== O || P.content === l ? P : (h = !0, { ...P, content: l, dirty: !0 }))), h && dt(O);
  }, [O, dt]), yn = N(async () => {
    if (Y)
      try {
        const l = await An(Y);
        q((h) => h.map((y) => y.id === l.id ? l : y)), j(`Saved ${l.name}`);
      } catch (l) {
        j(`Save failed: ${l.message}`);
      }
  }, [Y]), as = N(async (l) => {
    const h = f.find((P) => P.id === l);
    if (!h) return null;
    const y = await An(h);
    return q((P) => P.map((Q) => Q.id === y.id ? y : Q)), y;
  }, [f]), ba = N(async () => {
    const l = f.filter((h) => h.dirty);
    if (l.length === 0) {
      j("No dirty files to save");
      return;
    }
    try {
      const h = await Promise.all(l.map((P) => An(P))), y = new Map(h.map((P) => [P.id, P]));
      q((P) => P.map((Q) => y.get(Q.id) ?? Q)), ye(null), j(`Saved ${h.length} dirty file${h.length === 1 ? "" : "s"}`);
    } catch (h) {
      j(`Save all failed: ${h.message}`);
    }
  }, [f]), gn = N((l) => {
    const h = f.find((y) => y.id === l);
    if (h?.dirty && !window.confirm(`${h.name} has unsaved changes. Close without saving?`)) {
      j(`Close canceled — ${h.name} has unsaved changes`);
      return;
    }
    k((y) => {
      const P = y.filter((Q) => Q !== l);
      return O === l && (x(P.at(-1) ?? null), ve(null)), P;
    });
  }, [O, f]), rs = N(() => {
    jn(Oe, "close all editors") && (k([]), x(null), da(null), ee(!1), j("Closed all editors"));
  }, [Oe]), wn = N(() => {
    const l = f.filter((y) => y.name.startsWith("Untitled")).length + 1, h = {
      id: `untitled-${Date.now()}`,
      name: `Untitled-${l}`,
      path: `Untitled-${l}.md`,
      language: "markdown",
      content: `# Untitled
`,
      dirty: !0,
      source: "generated"
    };
    q((y) => [...y, h]), we(h);
  }, [f, we]), tt = N(async (l) => {
    const h = [...Pt.parts];
    if (ke !== "indexed-project")
      return Ye([]), h;
    const y = await Wo(
      e,
      _e.snapshot,
      l,
      Le,
      { limit: 8, maxChars: 12e3, includeDirty: !0 },
      _e.staleReport
    ), P = y.hits;
    return Ye(P), P.length === 0 ? (j(_e.snapshot.chunks.length === 0 ? "Project index is empty — open a folder with readable files first" : "Project index found no matching context for this prompt"), h) : (y.mode !== "hybrid" && y.reason ? j(y.reason) : j(`${P.length} project context hit${P.length === 1 ? "" : "s"} · ${y.reason ?? "hybrid retrieval"}`), [
      ...h,
      ...Vo(P)
    ]);
  }, [Pt.parts, ke, Le, e, _e]), kn = N(() => {
    if (!Y && Ie.length === 0) {
      j("Open a file before asking Atomek to synthesize an AI artifact");
      return;
    }
    const h = [
      `Create a polished Markdown artifact from ${Y?.path ?? `${Ie.length} open editors`}.`,
      "Use the open editor context already attached by Atomek.",
      "Prefer an actionable structure: summary, key findings, risks, and next steps.",
      "Do not invent missing facts. Do not use provider-specific tools or model assumptions."
    ].join(" ");
    F(!0), E("chat"), j("Asking Atomek to synthesize an AI artifact…"), (async () => {
      const y = await tt(h), P = await ne.askAgent(h, { requestContext: y });
      !P || P.status === "error" || ne.createArtifact({
        messageId: P.id,
        title: `AI synthesis — ${Y?.name ?? "open workspace"}`,
        kind: "markdown",
        body: P.body
      }).then(() => {
        F(!0), E("outputs"), Z(!0), V("output");
      });
    })();
  }, [Y, ne, tt, Ie.length]), xn = N(async (l) => {
    if (l.kind === "folder") {
      const y = f.filter((P) => P.path === l.path || P.path.startsWith(`${l.path}/`));
      if (fe?.name === l.name || y.length > 0) {
        y.length > 0 && (Te((P) => P?.name === l.name ? P : { name: l.name, files: y }), q((P) => Jt(P, y))), b("explorer"), v(!0), x(null), ee(!1), et({ ...l, at: Date.now() }), j(`Opened recent folder ${l.name}`);
        return;
      }
    }
    const h = f.find((y) => y.path === l.path || y.name === l.name);
    if (h) {
      we(h), j(`Opened recent ${h.name}`);
      return;
    }
    if (!l.handleKey) {
      j("Recent item has no stored browser permission. Use Open File or Open Folder once, then Atomek can restore it.");
      return;
    }
    try {
      if (l.kind === "folder") {
        const Q = await On(l.handleKey);
        if (!Q || !await zn(Q, "readwrite")) {
          j("Browser permission expired for this folder. Click Open Folder and pick it once to refresh the recent handle.");
          return;
        }
        const pe = await Pa(Q);
        xe(l.handleKey), Te(pe), q(pe.files), k([]), x(null), ee(!1), et({ ...l, at: Date.now() }), j(`Reopened ${pe.name} (${pe.files.length} text files indexed)`);
        return;
      }
      const y = await On(l.handleKey);
      if (!y || !await zn(y, "readwrite")) {
        j("Browser permission expired for this file. Click Open File and pick it once to refresh the recent handle.");
        return;
      }
      const P = await bi([y]);
      if (!P[0]) {
        j(`Recent file is not readable text: ${l.name}`);
        return;
      }
      q((Q) => Jt(Q, P)), we(P[0]), et({ ...l, at: Date.now() }), j(`Reopened ${P[0].name}`);
    } catch (y) {
      j(`Open recent failed: ${y instanceof Error ? y.message : String(y)}`);
    }
  }, [f, fe?.name, we, et]), Fe = N((l) => {
    F(!0), E("chat"), se(""), (async () => {
      const h = await tt(l);
      await ne.askAgent(l, { requestContext: h });
    })();
  }, [ne, tt]), va = N(async (l) => {
    if (!e.skills?.get) {
      j("Tytus skill registry is not available in this host build");
      return;
    }
    try {
      const h = await e.skills.get(l.id), y = h.body.length > 4500 ? `${h.body.slice(0, 4500)}

[Skill pack clipped by Atomek. Ask for the full pack if needed.]` : h.body, P = [
        `Use Tytus skill "${h.title}" (${h.id}).`,
        `Driver: ${h.driver}. Source: ${h.source}. Status: ${h.status}.`,
        "Follow these instructions only as capability context. Do not execute shell commands unless the user explicitly asks and Tytus host allows it.",
        y
      ].join(`

`);
      se((Q) => [Q.trim(), P].filter(Boolean).join(`

`)), F(!0), E("chat"), j(`Attached skill ${h.title} to chat input`);
    } catch (h) {
      j(`Failed to attach skill: ${h instanceof Error ? h.message : String(h)}`);
    }
  }, [e.skills]), ss = N((l) => {
    const h = ne.messages.findIndex((P) => P.id === l.id), y = ne.messages.slice(0, h < 0 ? void 0 : h).reverse().find((P) => P.role === "user");
    if (!y?.body.trim()) {
      j("No previous user prompt to regenerate from");
      return;
    }
    (async () => {
      const P = await tt(y.body);
      await ne.askAgent(y.body, { requestContext: P });
    })();
  }, [ne, tt]), is = N((l) => {
    ne.createArtifact({
      messageId: l.id,
      title: l.body.split(`
`).find(Boolean)?.replace(/^#+\s*/, "").slice(0, 80) || "Atomek answer",
      kind: "markdown",
      body: l.body
    }).then(() => {
      E("outputs"), Z(!0), V("output");
    });
  }, [ne]), os = N((l) => {
    ne.remember({
      messageId: l.id,
      title: l.body.split(`
`).find(Boolean)?.replace(/^#+\s*/, "").slice(0, 80) || "Atomek memory",
      body: l.body
    });
  }, [ne]), cs = N(() => {
    if (!Y) {
      j("No active file to save as AI artifact");
      return;
    }
    ne.createArtifact({
      title: Y.path,
      kind: Y.language === "markdown" ? "markdown" : "report",
      body: Y.content
    }).then(() => {
      F(!0), E("outputs"), Z(!0), V("output");
    });
  }, [Y, ne]), Pn = N((l) => {
    const h = Wn(f, Mn(l.title || l.kind)), y = {
      id: `artifact-file-${l.id}-${Date.now()}`,
      name: h,
      path: h,
      language: "markdown",
      content: l.body,
      dirty: !0,
      source: "generated"
    };
    q((P) => [...P, y]), we(y), j(`Opened ${l.title} as editable file`);
  }, [f, we]), Nn = N(() => {
    const l = window.prompt("Check command/name (manual capture only; Atomek will not execute it)");
    if (l === null) return;
    const h = l.trim();
    if (!h) {
      j("Manual check capture canceled — command/name was empty");
      return;
    }
    const y = window.prompt("Paste check output/result");
    if (y === null) return;
    const P = [
      "# Manual check result",
      "",
      `- Captured: ${(/* @__PURE__ */ new Date()).toISOString()}`,
      `- Command/name: \`${h.replace(/`/g, "\\`")}\``,
      "- Execution: manual user-provided output; Atomek did not run a shell command.",
      "",
      "```text",
      y,
      "```"
    ].join(`
`), Q = {
      id: `manual-check-${Date.now()}`,
      title: `Manual check — ${h.slice(0, 80)}`,
      kind: "report",
      body: P,
      createdAt: Date.now(),
      source: "local"
    };
    A((pe) => [Q, ...pe]), Z(!0), V("output"), j(`Captured manual check: ${h}`);
  }, []), ut = N((l, h) => {
    const y = tc({
      body: h,
      files: f,
      sourceTitle: l,
      activeFile: Y,
      versions: be
    }), P = {
      sourceTitle: y.sourceTitle,
      edits: y.edits.map(Wl),
      skipped: y.skipped
    };
    return y.edits.length > 1 ? (B(P), H(null), j(`Previewing AI workspace patch for ${y.edits.length} files`), !0) : y.edits.length === 1 ? (H(P.edits[0]), B(null), j(`Previewing AI patch for ${P.edits[0].fileName}`), !0) : (j(y.skipped.length > 0 ? `No applicable edit found. ${y.skipped.slice(0, 2).join(" · ")}` : "No fenced replacement block or applicable unified diff found. Ask Atomek for an edit again."), !1);
  }, [Y, be, f]), ya = N((l, h) => {
    const y = {
      id: `local-job-${Date.now()}`,
      title: l,
      kind: "report",
      body: h,
      createdAt: Date.now(),
      source: "local"
    };
    A((P) => [y, ...P]), F(!0), E("outputs"), Z(!0), V("output"), j(`Saved local job output: ${l}`), (h.includes("```diff") || h.includes("--- a/")) && ut(l, h);
  }, [ut]), ls = N(() => {
    const l = ce.trim();
    if (!l) return;
    se("");
    const h = Tl(l);
    (async () => {
      const y = await tt(l), P = await ne.askAgent(l, { requestContext: y });
      if (!h || !P || P.status === "error") return;
      ut(P.body.split(`
`).find(Boolean)?.replace(/^#+\s*/, "").slice(0, 80) || "Atomek edit", P.body) || j("Edit request answered without a patch. Use Generate patch / Edit to request an applicable diff.");
    })();
  }, [ne, tt, ce, ut]), ds = N(() => {
    te && (U(null), Fe(zl(te)));
  }, [Fe, te]), Tt = N((l) => {
    const h = vc(f, l);
    un(h);
    const y = h.commands[0]?.command ?? "";
    jt(y), mn(""), pn(""), la("failed"), Z(!0), V("terminal"), j(h.commands.length > 0 ? `Manual check ready: copy ${h.commands[0].command}` : "Manual check ready: enter a check command to copy");
  }, [f]), us = N((l) => {
    l.trim() && (gt(l), jt(l), j(`Manual check command copied: ${l}`));
  }, []), hs = N(() => {
    un((l) => {
      if (!l) return l;
      const h = yc(l, hn), y = h.commands.at(-1)?.command ?? "";
      return y && jt(y), h;
    }), mn("");
  }, [hn]), ms = N(() => {
    if (!Ve || !Ht.trim()) {
      j("Select or enter a manual check command before capturing output");
      return;
    }
    const l = Ht.trim();
    un((h) => h && gc(h, l, Mt, fn)), pn(""), j(`Captured manual check result: ${l} (${Mt})`);
  }, [fn, Ht, Ve, Mt]), fs = N(() => {
    Ve && Fe(wc(Ve));
  }, [Fe, Ve]), ps = N(() => {
    if (!C) return;
    const l = f.find((h) => h.id === C.fileId);
    if (!l) {
      j(`Cannot apply edit — ${C.fileName} is no longer open`), H(null);
      return;
    }
    l.content !== C.originalContent && !window.confirm(`${C.fileName} changed after the preview was created. Apply the AI edit anyway?`) || (q((h) => h.map((y) => y.id === C.fileId ? { ...y, content: C.proposedContent, dirty: !0 } : y)), dt(C.fileId), ye(`AI edit applied to ${l.name}. Save All to persist it to disk.`), H(null), Tt(`AI edit applied to ${l.name}`), j(`Applied AI edit to ${l.name} — unsaved; manual check ready`));
  }, [dt, f, C, Tt]), bs = N(() => {
    if (!C) return;
    const l = Wn(f, `${Mn(C.fileName)}-ai-edit`), h = {
      id: `pending-edit-${C.fileId}-${Date.now()}`,
      name: l,
      path: l,
      language: Y?.language ?? "markdown",
      content: C.proposedContent,
      dirty: !0,
      source: "generated"
    };
    q((y) => [...y, h]), we(h), H(null), j(`Opened proposed edit as ${l}`);
  }, [Y?.language, f, we, C]), vs = N(() => {
    if (!D) return;
    const l = new Map(D.edits.map((y) => [y.fileId, y]));
    f.some((y) => {
      const P = l.get(y.id);
      return P && y.content !== P.originalContent;
    }) && !window.confirm("One or more files changed after the workspace patch preview was created. Apply anyway?") || (q((y) => y.map((P) => {
      const Q = l.get(P.id);
      return Q ? { ...P, content: Q.proposedContent, dirty: !0 } : P;
    })), l.forEach((y, P) => dt(P)), ye(`AI workspace patch applied to ${l.size} file${l.size === 1 ? "" : "s"}. Save All to persist changes.`), B(null), Tt(`AI workspace patch applied to ${l.size} file${l.size === 1 ? "" : "s"}`), j(`Applied AI workspace patch to ${l.size} file${l.size === 1 ? "" : "s"} — unsaved; manual check ready`));
  }, [dt, f, D, Tt]), ys = N(() => {
    if (!D) return;
    const l = [...f], h = D.edits.map((y) => {
      const P = Wn(l, `${Mn(y.fileName)}-ai-edit`), Q = f.find((st) => st.id === y.fileId), pe = {
        id: `workspace-patch-${y.fileId}-${Date.now()}-${P}`,
        name: P,
        path: P,
        language: Q?.language ?? "markdown",
        content: y.proposedContent,
        dirty: !0,
        source: "generated"
      };
      return l.push(pe), pe;
    });
    q((y) => [...y, ...h]), h[0] && we(h[0]), B(null), j(`Opened ${h.length} proposed edit file${h.length === 1 ? "" : "s"}`);
  }, [f, we, D]), ga = N((l) => {
    if (l === "explain") {
      Fe("Explain the active file. Focus on purpose, structure, risks, and next useful edits.");
      return;
    }
    if (l === "improve") {
      Fe("Review the active file and propose the smallest concrete improvements. Include exact snippets if useful.");
      return;
    }
    if (l === "plan") {
      Fe("Create an implementation plan from the open editor context. Be specific, ordered, and call out blockers.");
      return;
    }
    if (l === "edit") {
      Fe("Edit the active file or open workspace files. Prefer one git-style unified diff in a fenced diff block, with paths matching opened files. If editing one file, a complete fenced replacement is also OK. Do not use provider-specific tools or model assumptions.");
      return;
    }
    Fe("Draft a concrete Markdown artifact from the open editor context. Make it ready to save as an output.");
  }, [Fe]);
  return oe(() => {
    const l = (h) => {
      (h.metaKey || h.ctrlKey) && (h.key.toLowerCase() === "o" && (h.preventDefault(), Nt()), h.key.toLowerCase() === "s" && (h.preventDefault(), yn()), h.key.toLowerCase() === "w" && (h.preventDefault(), O && gn(O)), h.key.toLowerCase() === "b" && (h.preventDefault(), v((P) => !P)), h.shiftKey && h.key.toLowerCase() === "f" && (h.preventDefault(), b("search"), v(!0)), (h.key.toLowerCase() === "k" || h.key.toLowerCase() === "p") && (h.preventDefault(), Me(!0)));
    };
    return window.addEventListener("keydown", l), () => window.removeEventListener("keydown", l);
  }, [O, gn, Nt, yn]), oe(() => {
    const l = (h) => {
      Oe.length !== 0 && (h.preventDefault(), h.returnValue = "");
    };
    return window.addEventListener("beforeunload", l), () => window.removeEventListener("beforeunload", l);
  }, [Oe.length]), oe(() => {
    Oe.length === 0 && ye(null);
  }, [Oe.length]), oe(() => {
    const l = n.current;
    if (!l) return;
    const h = () => i(Math.round(l.getBoundingClientRect().width));
    h();
    const y = new ResizeObserver(h);
    return y.observe(l), () => y.disconnect();
  }, []), oe(() => {
    if (!r) return;
    const l = Hn(r);
    S((h) => Et(h, l.primaryMin, l.primaryMax)), me((h) => Et(h, l.secondaryMin, l.secondaryMax));
  }, [r]), oe(() => {
    const l = { primaryVisible: p, primaryWidth: g, secondaryVisible: J, secondaryWidth: ae, markdownPreviewVisible: z };
    localStorage.setItem(Xr, JSON.stringify(l));
  }, [z, p, g, J, ae]), oe(() => {
    if (!o.folder?.handleKey) return;
    let l = !1;
    return (async () => {
      try {
        const h = await On(o.folder?.handleKey ?? "");
        if (!h || !await zn(h, "readwrite") || l) return;
        const y = await Pa(h);
        if (l) return;
        xe(o.folder?.handleKey ?? null), Te(y), q((P) => Yl(P, y.files)), k((P) => P.filter((Q) => y.files.some((pe) => pe.id === Q) || f.some((pe) => pe.id === Q))), j(`Restored ${y.name} (${y.files.length} text files indexed)`);
      } catch (h) {
        l || j(`Workspace restore failed: ${h instanceof Error ? h.message : String(h)}`);
      }
    })(), () => {
      l = !0;
    };
  }, []), oe(() => {
    ql({
      activity: m,
      folder: fe ? { name: fe.name, handleKey: He } : null,
      files: El(f),
      openEditorIds: G,
      activeFileId: O,
      query: L,
      chatInput: ce,
      welcomeClosed: ge,
      secondaryTab: M,
      bottomPanelVisible: W,
      bottomPanelTab: K,
      recent: xt
    });
  }, [m, O, K, W, ce, He, f, fe, G, L, xt, M, ge]), oe(() => {
    localStorage.setItem(Or, JSON.stringify(Le));
  }, [Le]), /* @__PURE__ */ u(
    "div",
    {
      ref: n,
      className: `workbench-workbench ${p ? "" : "no-primary"} ${J ? "" : "no-secondary"} ${W ? "has-bottom-panel" : ""}`,
      "data-app": "workbench-vscode-base",
      style: { "--workbench-primary-width": `${g}px`, "--workbench-secondary-width": `${ae}px` },
      children: [
        /* @__PURE__ */ a(
          cl,
          {
            active: m,
            setActive: (l) => {
              if (l === m && p) {
                v(!1);
                return;
              }
              b(l), v(!0);
            },
            togglePrimary: () => v((l) => !l),
            openSettings: Vt,
            settingsActive: Wt
          }
        ),
        p && /* @__PURE__ */ u("div", { className: "workbench-primary-region", children: [
          /* @__PURE__ */ a(
            ll,
            {
              host: e,
              activity: m,
              folder: fe,
              files: m === "search" ? f : Br,
              openEditors: Ie,
              activeFileId: O,
              query: L,
              setQuery: R,
              openFile: Nt,
              openFolder: vn,
              openWorkbenchFile: we,
              newFile: wn,
              recent: xt,
              reopenRecent: xn,
              setStatus: j,
              hasFsAccess: mi(),
              attachSkillToChat: va,
              saveLocalJobOutput: ya,
              activeFile: Y
            }
          ),
          /* @__PURE__ */ a("div", { className: "workbench-primary-resizer", onPointerDown: _r, title: t("shell.resizeExplorer") })
        ] }),
        /* @__PURE__ */ u("main", { className: "workbench-editor-area", children: [
          /* @__PURE__ */ a("button", { className: "workbench-command-center", onClick: () => Me(!0), children: t("app.workspace") }),
          /* @__PURE__ */ u("section", { className: "workbench-editor-stack", children: [
            /* @__PURE__ */ a(
              ml,
              {
                openEditors: Ie,
                activeFileId: O,
                showWelcome: bn,
                settingsOpen: ie,
                settingsActive: Wt,
                setActiveFileId: x,
                closeEditor: gn,
                saveFile: (l) => {
                  as(l);
                },
                closeWelcome: () => ee(!0),
                openSettings: Vt,
                closeSettings: pa,
                secondaryVisible: J,
                toggleSecondary: () => F((l) => !l),
                canPreview: Y?.language === "markdown",
                previewVisible: z,
                togglePreview: () => I((l) => !l)
              }
            ),
            /* @__PURE__ */ a(hl, { file: Y, folder: fe, showWelcome: bn }),
            Ce && Oe.length > 0 ? /* @__PURE__ */ u("div", { className: "workbench-ai-dirty-banner", children: [
              /* @__PURE__ */ a("span", { children: Ce }),
              /* @__PURE__ */ a("button", { onClick: () => {
                ba();
              }, children: t("shell.saveAll") }),
              /* @__PURE__ */ a("button", { onClick: () => ye(null), title: t("shell.dismiss"), children: /* @__PURE__ */ a(qe, { size: 13 }) })
            ] }) : null,
            /* @__PURE__ */ a("div", { className: "workbench-editor-content", children: Y ? /* @__PURE__ */ u("div", { className: Y.language === "markdown" && z ? "workbench-editor-split" : "workbench-editor-single", children: [
              /* @__PURE__ */ a("div", { className: "workbench-editor-pane", children: /* @__PURE__ */ a(xs, { fallback: /* @__PURE__ */ a("div", { className: "workbench-empty-pane", children: t("shell.loadingEditor") }), children: /* @__PURE__ */ a(
                Jc,
                {
                  file: Y,
                  revealLine: Jr,
                  onChange: ns,
                  onCursorChange: re,
                  onSelectionChange: ve,
                  onSave: () => {
                    yn();
                  }
                },
                Y.id
              ) }) }),
              Y.language === "markdown" && z && /* @__PURE__ */ a(wl, { content: Y.content })
            ] }) : Wt ? /* @__PURE__ */ a(
              Ll,
              {
                host: e,
                chatSettings: Le,
                onChange: le,
                onClose: pa
              }
            ) : bn ? /* @__PURE__ */ a(bl, { host: e, openFile: Nt, openFolder: vn, newFile: wn, recent: xt, reopenRecent: xn, setStatus: j, openControlTower: () => {
              b("computer"), v(!0);
            }, openChat: () => {
              E("chat"), F(!0);
            }, openEmbeddedDoc: es }) : /* @__PURE__ */ u("div", { className: "workbench-no-editor", children: [
              /* @__PURE__ */ a(on, { size: 34 }),
              /* @__PURE__ */ a("p", { children: t("shell.noEditor") }),
              /* @__PURE__ */ a("button", { className: "workbench-button-subtle", onClick: () => ee(!1), children: t("shell.showAgentTeam") })
            ] }) }),
            W && /* @__PURE__ */ a(
              xl,
              {
                tab: K,
                setTab: V,
                outputs: rt,
                clearOutputs: () => A([]),
                deleteArtifact: (l) => {
                  ne.deleteArtifact(l);
                },
                runAiSynthesis: kn,
                captureManualCheck: Nn,
                openOutputAsFile: Pn,
                manualCheckSession: Ve,
                manualCheckCommandInput: hn,
                setManualCheckCommandInput: mn,
                manualCheckOutputInput: fn,
                setManualCheckOutputInput: pn,
                manualCheckSelectedCommand: Ht,
                setManualCheckSelectedCommand: jt,
                manualCheckStatus: Mt,
                setManualCheckStatus: la,
                copyManualCheckCommand: us,
                addManualCheckCommand: hs,
                recordManualCheckResult: ms,
                askAgentFromManualChecks: fs,
                onClose: () => Z(!1)
              }
            )
          ] })
        ] }),
        J && /* @__PURE__ */ a(
          Pl,
          {
            tab: M,
            setTab: E,
            chatInput: ce,
            setChatInput: se,
            chatMessages: ne.messages,
            chatThread: ne.thread,
            chatThreads: ne.threads,
            askAgent: ls,
            stopChat: ne.stopChat,
            regenerateMessage: ss,
            newChat: () => {
              ne.newChat();
            },
            selectThread: (l) => {
              ne.selectThread(l);
            },
            renameThread: (l, h) => {
              ne.renameThread(l, h);
            },
            deleteThread: (l) => {
              ne.deleteThread(l);
            },
            saveMessageAsArtifact: is,
            rememberMessage: os,
            previewEditFromMessage: (l) => ut(l.body.split(`
`).find(Boolean)?.replace(/^#+\s*/, "").slice(0, 80) || "Atomek answer", l.body),
            runQuickPrompt: ga,
            pendingPatchPrompt: te,
            generatePatchPrompt: ds,
            workspaceFileCount: f.length,
            aiStatus: ne.aiStatus,
            chatSettings: Le,
            chatTargets: $,
            selectedChatTarget: fa,
            selectChatTarget: Gr,
            openSettings: Vt,
            busy: ne.busy,
            memoryHitCount: ne.memoryHits.length,
            outputs: rt,
            runAiSynthesis: kn,
            captureManualCheck: Nn,
            openOutputAsFile: Pn,
            previewEditFromOutput: (l) => ut(l.title, l.body),
            canPreviewEdit: f.length > 0,
            clearOutputs: () => A([]),
            deleteArtifact: (l) => {
              ne.deleteArtifact(l);
            },
            host: e,
            setStatus: j,
            activeFile: Y,
            openEditors: Ie,
            attachSkillToChat: va,
            saveLocalJobOutput: ya,
            contextScope: ke,
            setContextScope: Be,
            contextAttachments: Ur,
            removeContextAttachment: Qr,
            revealContextAttachment: ts,
            projectIndexSummary: `${_e.snapshot.files.length} files · ${_e.snapshot.chunks.length} chunks`,
            projectIndexStale: _e.isStale,
            refreshProjectIndex: () => {
              const l = _e.refresh(f);
              j(`Project index refreshed: ${l.files.length} files · ${l.chunks.length} chunks`);
            },
            onResizeStart: $r,
            onClose: () => F(!1)
          }
        ),
        Ae && /* @__PURE__ */ a(
          gl,
          {
            query: Ge,
            setQuery: d,
            files: f,
            activeFile: Y,
            commands: [
              { label: "File: New File", detail: "Create an untitled Markdown file", run: wn },
              { label: "File: Open File...", detail: "Open one or more local files", run: () => {
                Nt();
              } },
              { label: "File: Open Folder...", detail: "Open a local folder with browser permission", run: () => {
                vn();
              } },
              ...xt.map((l) => ({ label: `File: Open Recent — ${l.name}`, detail: l.path, run: () => {
                xn(l);
              } })),
              { label: "File: Save All", detail: `${Oe.length} dirty file${Oe.length === 1 ? "" : "s"}`, run: () => {
                ba();
              }, disabled: Oe.length === 0 },
              { label: "File: Close All Editors", detail: `${Ie.length} open editor${Ie.length === 1 ? "" : "s"}`, run: rs, disabled: Ie.length === 0 },
              { label: "Search: Find in Files", detail: "Open the VS Code-style search side bar", run: () => {
                b("search"), v(!0);
              } },
              { label: "Help: Show Agent Team", detail: "Open the Atomek Resource Fabric landing page", run: () => {
                x(null), ee(!1), b("computer"), v(!0);
              } },
              { label: "View: Toggle Primary Side Bar", detail: p ? "Hide Explorer side bar" : "Show Explorer side bar", run: () => v((l) => !l) },
              { label: "View: Toggle Chat Panel", detail: J ? "Hide right AI side bar" : "Show right AI side bar", run: () => F((l) => !l) },
              { label: "View: Toggle Bottom Panel", detail: W ? "Hide Problems/Output/Terminal panel" : "Show Problems/Output/Terminal panel", run: () => Z((l) => !l) },
              { label: "Atomek: Open Settings", detail: "Open settings as an editor tab", run: Vt },
              { label: "Checks: Open Manual Check Panel", detail: "Capture copy/paste check commands without host execution", run: () => Tt("Manual check requested from command palette") },
              { label: "View: Toggle Markdown Preview", detail: Y?.language === "markdown" ? "Show or hide Markdown preview split" : "Available for Markdown files", run: () => I((l) => !l), disabled: Y?.language !== "markdown" },
              { label: "Atomek: Create AI Synthesis", detail: Y || Ie.length > 0 ? "Ask AIL to produce a saved Markdown artifact" : "Open a file first", run: kn, disabled: !Y && Ie.length === 0 },
              { label: "AI: Explain Active File", detail: Y ? `Ask Cortex to explain ${Y.path}` : "Open a file first", run: () => Fe("Explain the active file. Focus on purpose, structure, risks, and next useful edits."), disabled: !Y },
              { label: "AI: Improve Active File", detail: Y ? `Ask Cortex for concrete edits to ${Y.path}` : "Open a file first", run: () => Fe("Review the active file and propose the smallest concrete improvements. Include exact snippets if useful."), disabled: !Y },
              { label: "AI: Draft Editable Replacement", detail: Y ? `Ask Cortex for a full-file replacement for ${Y.path}` : "Open a file first", run: () => ga("edit"), disabled: !Y },
              { label: "AI: Plan Workspace Work", detail: Ie.length > 0 ? "Use open editors as bounded context" : "Open files first", run: () => Fe("Create an implementation plan from the open editor context. Be specific and sequence the work."), disabled: Ie.length === 0 },
              { label: "AI: Save Active File as Artifact", detail: Y ? "Persist active file in host.ai artifacts" : "Open a file first", run: cs, disabled: !Y },
              { label: "AI: Capture Manual Check Output", detail: "Paste check output as an auditable local artifact; no shell execution", run: Nn },
              { label: "AI: Open Latest Artifact as File", detail: rt[0] ? `Create editable file from ${rt[0].title}` : "No outputs yet", run: () => rt[0] && Pn(rt[0]), disabled: rt.length === 0 }
            ],
            openWorkbenchFile: we,
            onClose: () => Me(!1)
          }
        ),
        C && /* @__PURE__ */ a(
          Xl,
          {
            edit: C,
            onApply: ps,
            onOpenAsFile: bs,
            onClose: () => H(null)
          }
        ),
        D && /* @__PURE__ */ a(
          Cl,
          {
            patch: D,
            onApply: vs,
            onOpenAsFiles: ys,
            onClose: () => B(null)
          }
        ),
        /* @__PURE__ */ a(Ml, { status: Zr, file: Y ?? Zc, cursor: X, fileCount: f.length, dirtyCount: Oe.length })
      ]
    }
  );
}
function cl({ active: e, setActive: t, togglePrimary: n, openSettings: r, settingsActive: i }) {
  const s = Ke();
  return /* @__PURE__ */ u("aside", { className: "workbench-activity-bar", "aria-label": s("activity.aria"), children: [
    /* @__PURE__ */ a("button", { className: "workbench-activity-brand", title: s("activity.toggleSidebar"), "aria-label": s("activity.toggleSidebar"), onClick: n, children: /* @__PURE__ */ a(Nr, { size: 30, variant: "cream" }) }),
    /* @__PURE__ */ a(Yt, { icon: /* @__PURE__ */ a(Za, { size: 25 }), label: s("activity.explorer"), active: e === "explorer", onClick: () => t("explorer") }),
    /* @__PURE__ */ a(Yt, { icon: /* @__PURE__ */ a(Ua, { size: 25 }), label: s("activity.search"), active: e === "search", onClick: () => t("search") }),
    /* @__PURE__ */ a(Yt, { icon: /* @__PURE__ */ a(Un, { size: 25 }), label: s("app.agentTeam"), active: e === "computer", onClick: () => t("computer") }),
    /* @__PURE__ */ a("div", { className: "workbench-activity-spacer" }),
    /* @__PURE__ */ a(Yt, { icon: /* @__PURE__ */ a(ii, { size: 23 }), label: s("app.settings"), active: i, onClick: r })
  ] });
}
function Yt({ icon: e, label: t, active: n, onClick: r }) {
  return /* @__PURE__ */ a("button", { className: `workbench-activity-button ${n ? "active" : ""}`, title: t, "aria-label": t, onClick: r, children: e });
}
function ll(e) {
  return e.activity === "search" ? /* @__PURE__ */ a(vl, { files: e.files, query: e.query, setQuery: e.setQuery, openWorkbenchFile: e.openWorkbenchFile, activeFileId: e.activeFileId }) : e.activity === "computer" ? /* @__PURE__ */ a(qr, { host: e.host, setStatus: e.setStatus, attachSkillToChat: e.attachSkillToChat, saveLocalJobOutput: e.saveLocalJobOutput, activeFile: e.activeFile, openEditors: e.openEditors }) : /* @__PURE__ */ a(dl, { ...e });
}
function dl(e) {
  const t = Ke(), n = !e.folder, r = ue(() => ul(e.files, e.folder?.name), [e.files, e.folder?.name]), [i, s] = T(() => /* @__PURE__ */ new Set());
  oe(() => {
    const c = Ir(r);
    s((m) => {
      const b = new Set(m);
      return c.forEach((p) => b.add(p)), b;
    });
  }, [r]);
  const o = N((c) => {
    s((m) => {
      const b = new Set(m);
      return b.has(c) ? b.delete(c) : b.add(c), b;
    });
  }, []);
  return /* @__PURE__ */ u("aside", { className: "workbench-sidebar", children: [
    /* @__PURE__ */ a("div", { className: "workbench-sidebar-title", children: t("explorer.title") }),
    /* @__PURE__ */ u("div", { className: "workbench-sidebar-scroll", children: [
      n ? /* @__PURE__ */ u(De, { children: [
        /* @__PURE__ */ a("p", { style: { fontWeight: 600, margin: "10px 0" }, children: t("explorer.noFolder") }),
        /* @__PURE__ */ a("p", { className: "workbench-muted", children: t("explorer.noFolderBody") }),
        /* @__PURE__ */ a("button", { className: "workbench-button-blue", onClick: e.openFolder, children: t("explorer.openFolder") }),
        /* @__PURE__ */ a("button", { className: "workbench-button-blue", onClick: e.openFile, children: t("explorer.openFile") }),
        /* @__PURE__ */ a("button", { className: "workbench-button-blue", onClick: () => e.recent[0] ? void e.reopenRecent(e.recent[0]) : e.setStatus(t("explorer.noRecentWorkspace")), children: t("explorer.openRecent") }),
        /* @__PURE__ */ a("p", { className: "workbench-muted", children: e.hasFsAccess ? t("explorer.fsAccess") : t("explorer.browserFallback") })
      ] }) : /* @__PURE__ */ u(De, { children: [
        /* @__PURE__ */ u("div", { className: "workbench-sidebar-actions", children: [
          /* @__PURE__ */ u("button", { className: "workbench-button-subtle", onClick: e.openFile, children: [
            /* @__PURE__ */ a(Qn, { size: 14 }),
            t("explorer.openFile")
          ] }),
          /* @__PURE__ */ u("button", { className: "workbench-button-subtle", onClick: e.openFolder, children: [
            /* @__PURE__ */ a($n, { size: 14 }),
            t("explorer.openFolder")
          ] })
        ] }),
        /* @__PURE__ */ a("input", { className: "workbench-input", value: e.query, onChange: (c) => e.setQuery(c.target.value), placeholder: t("explorer.searchFiles") }),
        /* @__PURE__ */ u("div", { className: "workbench-section-title", children: [
          /* @__PURE__ */ a(Xt, { size: 12 }),
          " ",
          t("explorer.openEditors")
        ] }),
        e.openEditors.length === 0 ? /* @__PURE__ */ a("p", { className: "workbench-muted", children: t("explorer.noOpenEditors") }) : e.openEditors.map((c) => /* @__PURE__ */ a(ca, { file: c, active: c.id === e.activeFileId, onOpen: () => e.openWorkbenchFile(c), label: c.name, detail: c.path }, c.id)),
        /* @__PURE__ */ u("div", { className: "workbench-section-title", children: [
          /* @__PURE__ */ a(Xt, { size: 12 }),
          " ",
          e.folder?.name ?? t("app.workspace")
        ] }),
        r.length === 0 ? /* @__PURE__ */ a("p", { className: "workbench-muted", children: t("explorer.noReadableFiles") }) : Fr(r, e.activeFileId, e.openWorkbenchFile, i, o)
      ] }),
      /* @__PURE__ */ a("div", { className: "workbench-section-title", children: t("explorer.recent") }),
      e.recent.length === 0 ? /* @__PURE__ */ a("p", { className: "workbench-muted", children: t("explorer.noRecentFolders") }) : e.recent.map((c) => /* @__PURE__ */ u("button", { className: "workbench-tree-row", onClick: () => {
        e.reopenRecent(c);
      }, title: c.path, children: [
        c.kind === "file" ? /* @__PURE__ */ a(Gn, { size: 14 }) : /* @__PURE__ */ a(Ka, { size: 14 }),
        /* @__PURE__ */ a("span", { className: "workbench-row-name", children: c.name })
      ] }, `${c.path}-${c.at}`))
    ] })
  ] });
}
function ca({ file: e, active: t, onOpen: n, basePath: r, depth: i = 0, label: s, detail: o }) {
  const c = r && e.path.startsWith(`${r}/`) ? e.path.slice(r.length + 1) : e.path, m = i || Math.max(0, c.split("/").length - 1);
  return /* @__PURE__ */ u("button", { className: `workbench-file-row ${t ? "active" : ""}`, style: { "--workbench-depth": m }, onClick: n, title: e.path, children: [
    /* @__PURE__ */ a(Gn, { size: 14 }),
    /* @__PURE__ */ u("span", { className: "workbench-row-text", children: [
      /* @__PURE__ */ a("span", { className: "workbench-row-name", children: s ?? c }),
      o ? /* @__PURE__ */ a("span", { className: "workbench-row-detail", children: o }) : null
    ] }),
    e.dirty && /* @__PURE__ */ a("span", { className: "workbench-row-meta", children: "●" })
  ] });
}
function ul(e, t) {
  const n = [], r = /* @__PURE__ */ new Map(), i = (o, c, m = n) => {
    const b = r.get(o);
    if (b) return b;
    const p = { name: c, path: o, children: [] };
    return r.set(o, p), m.push(p), p;
  };
  e.forEach((o) => {
    const c = t && o.path.startsWith(`${t}/`) ? o.path.slice(t.length + 1) : o.path, m = c.split("/").filter(Boolean);
    let b = n, p = "";
    m.slice(0, -1).forEach((v) => {
      p = p ? `${p}/${v}` : v, b = i(p, v, b).children;
    }), b.push({ name: m.at(-1) ?? o.name, path: c, file: o, children: [] });
  });
  const s = (o) => o.sort((c, m) => +!!c.file - +!!m.file || c.name.localeCompare(m.name)).map((c) => ({ ...c, children: s(c.children) }));
  return s(n);
}
function Ir(e) {
  return e.flatMap((t) => t.file ? [] : [t.path, ...Ir(t.children)]);
}
function Fr(e, t, n, r, i, s = 0) {
  return e.map((o) => {
    if (o.file)
      return /* @__PURE__ */ a(ca, { file: o.file, active: o.file.id === t, onOpen: () => n(o.file), depth: s, label: o.name }, o.file.id);
    const c = r.has(o.path);
    return /* @__PURE__ */ u("div", { children: [
      /* @__PURE__ */ u("button", { className: "workbench-folder-row", style: { "--workbench-depth": s }, onClick: () => i(o.path), title: c ? `Collapse ${o.name}` : `Expand ${o.name}`, children: [
        c ? /* @__PURE__ */ a(Xt, { size: 12 }) : /* @__PURE__ */ a(Xt, { className: "workbench-chevron-collapsed", size: 12 }),
        c ? /* @__PURE__ */ a($n, { size: 14 }) : /* @__PURE__ */ a(Ka, { size: 14 }),
        /* @__PURE__ */ a("span", { className: "workbench-row-name", children: o.name }),
        /* @__PURE__ */ a("span", { className: "workbench-row-meta", children: o.children.length })
      ] }),
      c ? Fr(o.children, t, n, r, i, s + 1) : null
    ] }, o.path);
  });
}
function hl({ file: e, folder: t, showWelcome: n }) {
  const r = Ke(), i = n ? [r("app.agentTeam")] : e?.path.split("/").filter(Boolean) ?? [], s = t && i[0] === t.name ? i.slice(1) : i;
  return /* @__PURE__ */ a("div", { className: "workbench-breadcrumb", children: s.length === 0 ? /* @__PURE__ */ a("span", { children: r("app.workspace") }) : s.map((o, c) => /* @__PURE__ */ u("span", { className: "workbench-breadcrumb-part", children: [
    c > 0 && /* @__PURE__ */ a("span", { className: "workbench-breadcrumb-sep", children: "›" }),
    o
  ] }, `${o}-${c}`)) });
}
function ml(e) {
  const t = Ke();
  return /* @__PURE__ */ u("div", { className: "workbench-tabs", children: [
    e.showWelcome && /* @__PURE__ */ u("button", { className: "workbench-tab active", children: [
      /* @__PURE__ */ a(on, { size: 15 }),
      /* @__PURE__ */ a("span", { className: "workbench-tab-name", children: t("app.agentTeam") }),
      /* @__PURE__ */ a("span", { className: "workbench-tab-close", role: "button", tabIndex: 0, onClick: (n) => {
        n.stopPropagation(), e.closeWelcome();
      }, children: /* @__PURE__ */ a(qe, { size: 13 }) })
    ] }),
    e.settingsOpen && /* @__PURE__ */ u("button", { className: `workbench-tab ${e.settingsActive ? "active" : ""}`, onClick: e.openSettings, title: t("tabs.atomekSettings"), children: [
      /* @__PURE__ */ a(Ba, { size: 15 }),
      /* @__PURE__ */ a("span", { className: "workbench-tab-name", children: t("tabs.atomekSettings") }),
      /* @__PURE__ */ a("span", { className: "workbench-tab-close", role: "button", tabIndex: 0, onClick: (n) => {
        n.stopPropagation(), e.closeSettings();
      }, children: /* @__PURE__ */ a(qe, { size: 13 }) })
    ] }),
    e.openEditors.map((n) => /* @__PURE__ */ u("button", { className: `workbench-tab ${n.id === e.activeFileId ? "active" : ""}`, onClick: () => e.setActiveFileId(n.id), title: n.path, children: [
      /* @__PURE__ */ a(Gn, { size: 15 }),
      /* @__PURE__ */ u("span", { className: "workbench-tab-name", children: [
        n.dirty && /* @__PURE__ */ a("span", { className: "workbench-dirty-dot", children: "●" }),
        n.name
      ] }),
      n.dirty && /* @__PURE__ */ a("span", { className: "workbench-tab-save", role: "button", tabIndex: 0, title: t("tabs.save"), onClick: (r) => {
        r.stopPropagation(), e.saveFile(n.id);
      }, children: t("tabs.save") }),
      /* @__PURE__ */ a("span", { className: "workbench-tab-close", role: "button", tabIndex: 0, onClick: (r) => {
        r.stopPropagation(), e.closeEditor(n.id);
      }, children: /* @__PURE__ */ a(qe, { size: 13 }) })
    ] }, n.id)),
    /* @__PURE__ */ a("div", { style: { flex: 1 } }),
    e.canPreview && /* @__PURE__ */ a("button", { className: `workbench-editor-action ${e.previewVisible ? "active" : ""}`, title: t("tabs.toggleMarkdownPreview"), onClick: e.togglePreview, children: /* @__PURE__ */ a(Bn, { size: 16 }) }),
    /* @__PURE__ */ a("button", { className: `workbench-editor-action ${e.secondaryVisible ? "active" : ""}`, title: t("tabs.toggleChat"), onClick: e.toggleSecondary, children: /* @__PURE__ */ a(ei, { size: 16 }) })
  ] });
}
function fl(e) {
  const t = e?.resources ?? [], n = (i) => t.filter((s) => s.kind === i).length, r = t.filter((i) => i.status === "ready" || i.status === "available").length;
  return [
    { label: "Pod agents", value: n("pod-agent"), detail: "OpenClaw + Hermes worker pods" },
    { label: "Local agents", value: n("local-cli"), detail: "Claude, OpenCode, Codex, pi, Kimi" },
    { label: "Shared folders", value: n("shared-folder"), detail: "garagetytus + mission handoff" },
    { label: "App skills", value: n("app-skill"), detail: "Blender, JULI3TA, Remotion, tools" },
    { label: "Ready resources", value: r, detail: "usable now" }
  ];
}
function pl(e) {
  const t = {
    "pod-agent": 0,
    "local-cli": 1,
    "shared-folder": 2,
    "app-skill": 3,
    "ail-route": 4,
    workspace: 5
  };
  return [...e?.resources ?? []].sort((n, r) => (t[n.kind] ?? 9) - (t[r.kind] ?? 9) || je(n).localeCompare(je(r))).slice(0, 8);
}
function bl({
  host: e,
  openFile: t,
  openFolder: n,
  newFile: r,
  recent: i,
  reopenRecent: s,
  setStatus: o,
  openControlTower: c,
  openChat: m,
  openEmbeddedDoc: b
}) {
  const [p, v] = T("Coordinate a Tytus mission across pods, local agents, shared folders, and app skills."), [g, S] = T(null), [M, E] = T([]), [J, F] = T(!1), [ae, me] = T(null), [W, Z] = T(null), [K, V] = T("pod-local"), z = N(async () => {
    F(!0), Z(null);
    try {
      const [k, O] = await Promise.all([
        e.resources?.list?.() ?? Promise.resolve(null),
        e.missions?.list?.().catch(() => []) ?? Promise.resolve([])
      ]);
      S(k ?? null), E(O), k && o(`Agent team loaded · ${k.resources.length} resources · ${k.warnings.length} warnings`);
    } catch (k) {
      const O = k instanceof Error ? k.message : String(k);
      Z(O), o(`Agent team resource load failed: ${O}`);
    } finally {
      F(!1);
    }
  }, [e.missions, e.resources, o]), I = N((k, O) => {
    try {
      e.windows.openOrFocus(k), o(`Opened ${O}`);
    } catch (x) {
      o(`Open ${O} failed: ${x instanceof Error ? x.message : String(x)}`);
    }
  }, [e.windows, o]);
  oe(() => {
    z();
  }, [z]);
  const ge = N(async () => {
    if (!e.missions?.create || !e.missions?.write) {
      o("Mission API unavailable in this Tytus host build"), c();
      return;
    }
    const k = p.trim() || "Coordinate a Tytus mission.", O = lt(k, g, K);
    F(!0);
    try {
      const x = await e.missions.create({
        title: `Atomek team mission — ${(/* @__PURE__ */ new Date()).toLocaleString()}`,
        goal: k
      }), L = {
        missionId: x.missionId,
        title: x.title,
        goal: x.goal,
        rootPath: x.rootPath,
        name: x.rootPath.split("/").pop() || x.missionId,
        source: "tray",
        teamPresetId: O
      }, R = {
        ts: (/* @__PURE__ */ new Date()).toISOString(),
        kind: "mission.control.created",
        message: "Mission created from Atomek agent-team home",
        data: { resourceCount: g?.resources.length ?? 0 }
      };
      await e.missions.write({
        rootPath: x.rootPath,
        files: [
          { path: "MISSION.md", content: tn(L, g, null, [], k, O) },
          { path: "MISSION.json", content: Vr(L, g, k, O) },
          { path: "RESOURCES.md", content: nn(g) },
          { path: "TASKS.md", content: Mr(Ot(k, g, O)) },
          { path: "HANDOFF.md", content: Wr(L) },
          { path: "INBOX.md", content: `# Mission inbox

Drop incoming agent notes, pod outputs, and shared-folder discoveries here.
` },
          { path: "OUTBOX.md", content: `# Mission outbox

Approved handoffs, final artifacts, and user-ready summaries go here.
` },
          { path: "AUDIT.jsonl", content: `${JSON.stringify(R)}
` },
          { path: "RUNS.jsonl", content: "" },
          { path: "runs/README.md", content: `# Mission runs

Local, pod, and app run transcripts land here.
` },
          { path: "outputs/README.md", content: `# Mission outputs

Final artifacts and generated files land here before handoff.
` },
          { path: "proposals/README.md", content: `# Mission proposals

Patch/write/publish proposals land here before approval.
` },
          { path: "approvals/README.md", content: `# Mission approvals

Approval and rejection decisions reference proposal files from here.
` },
          { path: "NEXT.md", content: ["# Next actions", "", "- Pick resources for the mission.", "- Break goal into task cards.", "- Dispatch local/pod/app runs through Atomek.", "- Review approvals before applying outputs.", ""].join(`
`) }
        ]
      }), me(x), Lt(L), o(`Mission created: ${x.rootPath}`), c();
    } catch (x) {
      o(`Mission create failed: ${x instanceof Error ? x.message : String(x)}`);
    } finally {
      F(!1);
    }
  }, [p, g, e.missions, c, o, K]), ee = fl(g), fe = el(g), Te = tl(g), He = pl(g), xe = g?.warnings ?? [], f = al(g), q = kt(g, lt(p, g, K)), G = Ot(p, g, q.id);
  return /* @__PURE__ */ u("div", { className: "workbench-welcome workbench-control-home", children: [
    /* @__PURE__ */ u("section", { className: "workbench-control-hero-main", children: [
      /* @__PURE__ */ u("div", { className: "workbench-brand-lockup", "aria-label": "Atomek", children: [
        /* @__PURE__ */ a(Nr, { size: 74, variant: "acid" }),
        /* @__PURE__ */ u("div", { children: [
          /* @__PURE__ */ a(Hc, { className: "workbench-brand-wordmark" }),
          /* @__PURE__ */ a("div", { className: "workbench-control-kicker", children: "Resource Fabric / Agent Team" })
        ] })
      ] }),
      /* @__PURE__ */ a("h1", { children: "Split the mission. Ship the build." }),
      /* @__PURE__ */ a("p", { children: "Atomek is the Tytus control surface for OpenClaw, Hermes, local AI agents, shared folders, pods, local apps, files, outputs, and approval-gated handoffs around one durable mission folder." }),
      /* @__PURE__ */ u("div", { className: "workbench-control-goal-row", children: [
        /* @__PURE__ */ a("textarea", { value: p, onChange: (k) => v(k.target.value), rows: 3, "aria-label": "Mission goal" }),
        /* @__PURE__ */ u("div", { className: "workbench-control-hero-actions", children: [
          /* @__PURE__ */ a("button", { className: "workbench-button-primary", onClick: () => {
            ge();
          }, disabled: J, children: "Start mission" }),
          /* @__PURE__ */ a("button", { className: "workbench-button-subtle", onClick: c, children: "Open team board" }),
          /* @__PURE__ */ a("button", { className: "workbench-button-subtle", onClick: m, children: "Open chat" })
        ] })
      ] }),
      /* @__PURE__ */ a("div", { className: "workbench-team-preset-strip", "aria-label": "Team presets", children: f.map((k) => /* @__PURE__ */ u(
        "button",
        {
          className: `workbench-team-preset-card ${k.id === q.id ? "active" : ""} ${k.readiness}`,
          onClick: () => V(k.id),
          children: [
            /* @__PURE__ */ u("div", { children: [
              /* @__PURE__ */ a("strong", { children: k.label }),
              /* @__PURE__ */ a("span", { children: k.readiness })
            ] }),
            /* @__PURE__ */ a("p", { children: k.summary }),
            /* @__PURE__ */ a("small", { children: k.assignments.map((O) => O.resourceLabel).slice(0, 3).join(" · ") })
          ]
        },
        k.id
      )) }),
      ae ? /* @__PURE__ */ u("div", { className: "workbench-control-success", children: [
        "Mission ready: ",
        /* @__PURE__ */ a("code", { children: ae.rootPath })
      ] }) : null,
      W ? /* @__PURE__ */ a("div", { className: "workbench-inline-error", children: W }) : null
    ] }),
    /* @__PURE__ */ u("section", { className: "workbench-control-grid", children: [
      /* @__PURE__ */ u("article", { className: "workbench-control-card wide", children: [
        /* @__PURE__ */ u("header", { children: [
          /* @__PURE__ */ a("strong", { children: "System inventory" }),
          /* @__PURE__ */ u("button", { className: "workbench-button-subtle", onClick: () => {
            z();
          }, disabled: J, children: [
            /* @__PURE__ */ a(bt, { size: 13 }),
            " ",
            J ? "Refreshing…" : "Refresh"
          ] })
        ] }),
        /* @__PURE__ */ a("div", { className: "workbench-control-metrics", children: ee.map((k) => /* @__PURE__ */ u("div", { className: "workbench-control-metric", children: [
          /* @__PURE__ */ a("strong", { children: k.value }),
          /* @__PURE__ */ a("span", { children: k.label }),
          /* @__PURE__ */ a("em", { children: k.detail })
        ] }, k.label)) })
      ] }),
      /* @__PURE__ */ u("article", { className: "workbench-control-card wide", children: [
        /* @__PURE__ */ u("header", { children: [
          /* @__PURE__ */ a("strong", { children: "Agent team" }),
          /* @__PURE__ */ a("span", { children: "OpenClaw + Hermes are first-class" })
        ] }),
        /* @__PURE__ */ a("div", { className: "workbench-agent-brand-grid", children: fe.map((k) => /* @__PURE__ */ u("div", { className: `workbench-agent-brand-card ${k.status.replace(/\s+/g, "-")}`, children: [
          /* @__PURE__ */ u("div", { children: [
            /* @__PURE__ */ a("strong", { children: k.label }),
            /* @__PURE__ */ a("span", { children: k.status })
          ] }),
          /* @__PURE__ */ a("b", { children: k.value }),
          /* @__PURE__ */ a("p", { children: k.detail })
        ] }, k.label)) })
      ] }),
      /* @__PURE__ */ u("article", { className: "workbench-control-card wide", children: [
        /* @__PURE__ */ u("header", { children: [
          /* @__PURE__ */ a("strong", { children: "Docs & Skills" }),
          /* @__PURE__ */ a("span", { children: "open a guide as a markdown tab, then ask chat about it" })
        ] }),
        /* @__PURE__ */ a("div", { className: "workbench-doc-grid", children: Ec.map((k) => /* @__PURE__ */ u("button", { className: "workbench-doc-card", onClick: () => b(k), title: k.summary, children: [
          /* @__PURE__ */ a("strong", { children: k.title }),
          /* @__PURE__ */ a("p", { children: k.summary }),
          /* @__PURE__ */ a("small", { children: k.tags.join(" · ") })
        ] }, k.id)) })
      ] }),
      /* @__PURE__ */ u("article", { className: "workbench-control-card wide", children: [
        /* @__PURE__ */ u("header", { children: [
          /* @__PURE__ */ a("strong", { children: "Tytus Resource Fabric" }),
          /* @__PURE__ */ a("span", { children: "local computer ↔ shared folders ↔ pods ↔ apps" })
        ] }),
        /* @__PURE__ */ a("div", { className: "workbench-fabric-flow", children: Te.map((k, O) => /* @__PURE__ */ u("div", { className: `workbench-fabric-node ${k.status.replace(/\s+/g, "-")}`, children: [
          /* @__PURE__ */ a("span", { children: O + 1 }),
          /* @__PURE__ */ a("strong", { children: k.label }),
          /* @__PURE__ */ a("em", { children: k.status }),
          /* @__PURE__ */ a("p", { children: k.detail })
        ] }, k.label)) }),
        /* @__PURE__ */ u("div", { className: "workbench-fabric-actions", children: [
          /* @__PURE__ */ a("button", { className: "workbench-button-subtle", onClick: () => I(Rt.sharedFiles, "Shared Files"), children: "Open shared files" }),
          /* @__PURE__ */ a("button", { className: "workbench-button-subtle", onClick: () => I(Rt.podInspector, "Pod Inspector"), children: "Open pods" }),
          /* @__PURE__ */ a("button", { className: "workbench-button-subtle", onClick: () => I(Rt.channels, "Channels"), children: "Open channels" }),
          /* @__PURE__ */ a("button", { className: "workbench-button-subtle", onClick: () => I(Rt.settings, "Agent Settings"), children: "Open agent settings" })
        ] })
      ] }),
      /* @__PURE__ */ u("article", { className: "workbench-control-card", children: [
        /* @__PURE__ */ a("header", { children: /* @__PURE__ */ a("strong", { children: "First missions" }) }),
        /* @__PURE__ */ a("button", { className: "workbench-control-preset", onClick: () => v("Review this repo with local OpenCode/Claude, then ask Codex or pi for an independent review. Save all transcripts and proposed patches in the mission folder."), children: "Review + patch repo" }),
        /* @__PURE__ */ a("button", { className: "workbench-control-preset", onClick: () => v("Ask OpenClaw for independent critique, use Hermes when allocated for deep planning/copy, then run local Claude/OpenCode/Codex for implementation. Keep shared context in the mission folder and approval-gate all edits."), children: "OpenClaw/Hermes + local agent" }),
        /* @__PURE__ */ a("button", { className: "workbench-control-preset", onClick: () => v("Coordinate creative output: script, audio, visuals, render assets, and final handoff through app skills and shared folders."), children: "Creative production" })
      ] }),
      /* @__PURE__ */ u("article", { className: "workbench-control-card", children: [
        /* @__PURE__ */ u("header", { children: [
          /* @__PURE__ */ a("strong", { children: "Resume" }),
          /* @__PURE__ */ a("span", { children: M.length ? `${M.length} missions` : "none yet" })
        ] }),
        M.length === 0 ? /* @__PURE__ */ a("p", { className: "workbench-muted", children: "Create a mission once; it stays in Tytus Home/Missions and can be resumed here." }) : null,
        M.slice(0, 4).map((k) => /* @__PURE__ */ u("button", { className: "workbench-control-preset", onClick: () => {
          Lt(k), c();
        }, title: k.rootPath, children: [
          k.title,
          /* @__PURE__ */ u("small", { children: [
            k.status ?? "active",
            " · ",
            k.taskCount ?? 0,
            " tasks · ",
            k.runCount ?? 0,
            " runs"
          ] })
        ] }, k.missionId))
      ] }),
      /* @__PURE__ */ u("article", { className: "workbench-control-card", children: [
        /* @__PURE__ */ a("header", { children: /* @__PURE__ */ a("strong", { children: "Workspace" }) }),
        /* @__PURE__ */ u("button", { className: "workbench-start-link", onClick: n, children: [
          /* @__PURE__ */ a($n, { size: 18 }),
          "Open workspace folder"
        ] }),
        /* @__PURE__ */ u("button", { className: "workbench-start-link", onClick: t, children: [
          /* @__PURE__ */ a(Za, { size: 18 }),
          "Open file"
        ] }),
        /* @__PURE__ */ u("button", { className: "workbench-start-link", onClick: r, children: [
          /* @__PURE__ */ a(Qn, { size: 18 }),
          "New mission note"
        ] }),
        i.length ? /* @__PURE__ */ u("div", { className: "workbench-control-recent", children: [
          /* @__PURE__ */ a("span", { children: "Recent" }),
          i.slice(0, 3).map((k) => /* @__PURE__ */ a("button", { onClick: () => {
            s(k);
          }, children: k.name }, `${k.path}-${k.at}`))
        ] }) : null
      ] }),
      /* @__PURE__ */ u("article", { className: "workbench-control-card wide", children: [
        /* @__PURE__ */ u("header", { children: [
          /* @__PURE__ */ a("strong", { children: "Resource graph" }),
          /* @__PURE__ */ a("span", { children: g ? `${g.resources.length} discovered` : "not loaded" })
        ] }),
        /* @__PURE__ */ u("div", { className: "workbench-control-resource-list", children: [
          He.length === 0 ? /* @__PURE__ */ a("p", { className: "workbench-muted", children: "No resource graph yet. Start Tytus tray beta30 or newer, then refresh." }) : null,
          He.map((k) => /* @__PURE__ */ u("div", { className: "workbench-control-resource", children: [
            /* @__PURE__ */ u("div", { children: [
              /* @__PURE__ */ a("strong", { children: je(k) }),
              /* @__PURE__ */ a("span", { children: oa(k) })
            ] }),
            /* @__PURE__ */ a("span", { className: `workbench-computer-pill ${k.status}`, children: k.status })
          ] }, k.id))
        ] }),
        xe.length ? /* @__PURE__ */ a("div", { className: "workbench-resource-warnings", children: xe.slice(0, 2).map((k) => /* @__PURE__ */ u("span", { children: [
          k.code,
          ": ",
          k.message
        ] }, `${k.code}-${k.resourceId ?? k.message}`)) }) : null
      ] }),
      /* @__PURE__ */ u("article", { className: "workbench-control-card wide", children: [
        /* @__PURE__ */ a("header", { children: /* @__PURE__ */ a("strong", { children: "Recommended task graph" }) }),
        /* @__PURE__ */ a("div", { className: "workbench-task-graph home", children: G.map((k, O) => /* @__PURE__ */ u("button", { className: `workbench-task-card ${k.status}`, onClick: () => v(k.prompt), children: [
          /* @__PURE__ */ a("span", { children: O + 1 }),
          /* @__PURE__ */ a("strong", { children: k.title }),
          /* @__PURE__ */ a("em", { children: k.resourceHint }),
          /* @__PURE__ */ a("small", { children: k.assignedResourceLabel })
        ] }, k.id)) })
      ] }),
      /* @__PURE__ */ u("article", { className: "workbench-control-card wide", children: [
        /* @__PURE__ */ a("header", { children: /* @__PURE__ */ a("strong", { children: "Control loop" }) }),
        /* @__PURE__ */ u("ol", { className: "workbench-control-loop", children: [
          /* @__PURE__ */ u("li", { children: [
            /* @__PURE__ */ a("b", { children: "Mission" }),
            /* @__PURE__ */ a("span", { children: "Goal + shared context folder." })
          ] }),
          /* @__PURE__ */ u("li", { children: [
            /* @__PURE__ */ a("b", { children: "Resources" }),
            /* @__PURE__ */ a("span", { children: "Pods, local agents, folders, app skills." })
          ] }),
          /* @__PURE__ */ u("li", { children: [
            /* @__PURE__ */ a("b", { children: "Tasks" }),
            /* @__PURE__ */ a("span", { children: "Plan, implement, review, render, validate." })
          ] }),
          /* @__PURE__ */ u("li", { children: [
            /* @__PURE__ */ a("b", { children: "Runs" }),
            /* @__PURE__ */ u("span", { children: [
              "Streams + transcripts saved under ",
              /* @__PURE__ */ a("code", { children: "runs/" }),
              "."
            ] })
          ] }),
          /* @__PURE__ */ u("li", { children: [
            /* @__PURE__ */ a("b", { children: "Approvals" }),
            /* @__PURE__ */ a("span", { children: "Diffs/artifacts applied only after preview." })
          ] })
        ] })
      ] })
    ] })
  ] });
}
function vl({ files: e, query: t, setQuery: n, openWorkbenchFile: r, activeFileId: i }) {
  const s = Ke(), o = ue(() => yl(e, t), [e, t]), c = ue(() => {
    const m = /* @__PURE__ */ new Map();
    return o.forEach((b) => m.set(b.file.id, [...m.get(b.file.id) ?? [], b])), Array.from(m.values()).slice(0, 50);
  }, [o]);
  return /* @__PURE__ */ u("aside", { className: "workbench-sidebar", children: [
    /* @__PURE__ */ a("div", { className: "workbench-sidebar-title", children: s("search.title") }),
    /* @__PURE__ */ u("div", { className: "workbench-sidebar-scroll", children: [
      /* @__PURE__ */ a("input", { className: "workbench-input", value: t, onChange: (m) => n(m.target.value), placeholder: s("search.placeholder"), autoFocus: !0 }),
      /* @__PURE__ */ u("div", { className: "workbench-section-title", children: [
        /* @__PURE__ */ a(on, { size: 12 }),
        " ",
        s("search.results")
      ] }),
      t.trim() ? c.length === 0 ? /* @__PURE__ */ a("p", { className: "workbench-muted", children: s("search.noMatches") }) : c.map((m) => {
        const b = m[0].file;
        return /* @__PURE__ */ u("div", { className: "workbench-search-group", children: [
          /* @__PURE__ */ a(ca, { file: b, active: b.id === i, onOpen: () => r(b) }),
          m.slice(0, 5).map((p) => /* @__PURE__ */ u("button", { className: "workbench-search-hit", onClick: () => r(b, p.lineNumber), title: p.line, children: [
            /* @__PURE__ */ a("span", { className: "workbench-search-line", children: p.lineNumber }),
            /* @__PURE__ */ a("span", { children: p.line })
          ] }, `${b.id}-${p.lineNumber}-${p.line}`)),
          m.length > 5 && /* @__PURE__ */ a("div", { className: "workbench-search-more", children: s("search.moreMatches", { count: m.length - 5 }) })
        ] }, b.id);
      }) : /* @__PURE__ */ a("p", { className: "workbench-muted", children: s("search.help") })
    ] })
  ] });
}
function yl(e, t) {
  const n = t.trim().toLowerCase();
  if (!n) return [];
  const r = [];
  return e.forEach((i) => {
    i.path.toLowerCase().includes(n) && r.push({ file: i, lineNumber: 1, line: i.path }), i.content.split(`
`).some((s, o) => s.toLowerCase().includes(n) ? (r.push({ file: i, lineNumber: o + 1, line: s.trim() || "(blank line)" }), r.length >= 200) : !1);
  }), r.slice(0, 200);
}
function gl(e) {
  const t = e.query.trim().toLowerCase(), n = e.files.slice(0, 80).map((i) => ({
    label: i.path,
    detail: `Open ${$a(i.language)} file`,
    run: () => e.openWorkbenchFile(i)
  })), r = [...e.commands, ...n].filter((i) => !t || i.label.toLowerCase().includes(t) || i.detail.toLowerCase().includes(t)).slice(0, 12);
  return oe(() => {
    const i = (s) => {
      s.key === "Escape" && e.onClose();
    };
    return window.addEventListener("keydown", i), () => window.removeEventListener("keydown", i);
  }, [e.onClose]), /* @__PURE__ */ a("div", { className: "workbench-command-overlay", role: "dialog", "aria-label": "Command Palette", children: /* @__PURE__ */ u("div", { className: "workbench-command-palette", children: [
    /* @__PURE__ */ a(
      "input",
      {
        className: "workbench-command-input",
        autoFocus: !0,
        value: e.query,
        onChange: (i) => e.setQuery(i.target.value),
        placeholder: "Type a command or file name...",
        onKeyDown: (i) => {
          i.key === "Enter" && r[0] && !r[0].disabled && (r[0].run(), e.onClose());
        }
      }
    ),
    /* @__PURE__ */ a("div", { className: "workbench-command-list", children: r.map((i) => /* @__PURE__ */ u(
      "button",
      {
        disabled: i.disabled,
        className: "workbench-command-item",
        onClick: () => {
          i.run(), e.onClose();
        },
        children: [
          /* @__PURE__ */ a("span", { children: i.label }),
          /* @__PURE__ */ a("small", { children: i.detail })
        ]
      },
      `${i.label}-${i.detail}`
    )) })
  ] }) });
}
function wl({ content: e }) {
  const t = ue(() => nr(e), [e]);
  return /* @__PURE__ */ u("aside", { className: "workbench-markdown-preview", children: [
    /* @__PURE__ */ u("div", { className: "workbench-preview-title", children: [
      /* @__PURE__ */ a(Bn, { size: 13 }),
      " Preview"
    ] }),
    /* @__PURE__ */ a("div", { className: "workbench-preview-body", dangerouslySetInnerHTML: { __html: t } })
  ] });
}
function kl(e) {
  const t = e.session ? kc(e.session) : "pending";
  return /* @__PURE__ */ u("div", { className: "workbench-manual-check-panel", children: [
    /* @__PURE__ */ a("p", { className: "workbench-muted", children: "Manual checks never execute host commands. Copy a command, run it yourself, then paste the result." }),
    e.session ? /* @__PURE__ */ u(De, { children: [
      /* @__PURE__ */ u("div", { className: "workbench-manual-check-head", children: [
        /* @__PURE__ */ a("strong", { children: "Manual edit-check loop" }),
        /* @__PURE__ */ a("span", { className: `workbench-check-status ${t}`, children: t }),
        /* @__PURE__ */ a("small", { children: e.session.reason })
      ] }),
      /* @__PURE__ */ u("div", { className: "workbench-manual-check-grid", children: [
        /* @__PURE__ */ u("section", { children: [
          /* @__PURE__ */ a("label", { children: "Check commands" }),
          e.session.commands.length === 0 ? /* @__PURE__ */ a("p", { className: "workbench-muted", children: "No project check scripts detected from open files. Add the command you want to run." }) : null,
          /* @__PURE__ */ a("div", { className: "workbench-check-command-list", children: e.session.commands.map((n) => /* @__PURE__ */ u(
            "button",
            {
              className: e.selectedCommand === n.command ? "active" : "",
              onClick: () => e.setSelectedCommand(n.command),
              title: n.path ?? n.source,
              children: [
                /* @__PURE__ */ a("span", { children: n.command }),
                /* @__PURE__ */ a("small", { children: n.source === "package-script" ? n.label : "manual" })
              ]
            },
            n.id
          )) }),
          /* @__PURE__ */ u("div", { className: "workbench-check-add-row", children: [
            /* @__PURE__ */ a(
              "input",
              {
                value: e.commandInput,
                onChange: (n) => e.setCommandInput(n.target.value),
                placeholder: "Add manual command to copy"
              }
            ),
            /* @__PURE__ */ a("button", { onClick: e.addCommand, disabled: !e.commandInput.trim(), children: "Add" })
          ] }),
          /* @__PURE__ */ a("button", { className: "workbench-button-primary", onClick: () => e.copyCommand(e.selectedCommand), disabled: !e.selectedCommand.trim(), children: "Copy selected command" })
        ] }),
        /* @__PURE__ */ u("section", { children: [
          /* @__PURE__ */ a("label", { children: "Paste result" }),
          /* @__PURE__ */ u("select", { value: e.status, onChange: (n) => e.setStatus(n.target.value), children: [
            /* @__PURE__ */ a("option", { value: "failed", children: "failed" }),
            /* @__PURE__ */ a("option", { value: "passed", children: "passed" }),
            /* @__PURE__ */ a("option", { value: "pending", children: "pending/manual note" })
          ] }),
          /* @__PURE__ */ a(
            "textarea",
            {
              value: e.outputInput,
              onChange: (n) => e.setOutputInput(n.target.value),
              placeholder: "Paste stdout/stderr or a short manual QA note. Nothing runs in Atomek.",
              rows: 6
            }
          ),
          /* @__PURE__ */ u("div", { className: "workbench-check-actions", children: [
            /* @__PURE__ */ a("button", { onClick: e.recordResult, disabled: !e.selectedCommand.trim(), children: "Capture result" }),
            /* @__PURE__ */ a("button", { className: "workbench-button-primary", onClick: e.askAgent, disabled: e.session.results.length === 0, children: "Ask Atomek to continue" })
          ] })
        ] })
      ] }),
      e.session.results.length > 0 ? /* @__PURE__ */ u("div", { className: "workbench-check-results", children: [
        /* @__PURE__ */ a("label", { children: "Captured results" }),
        e.session.results.map((n, r) => /* @__PURE__ */ u("article", { children: [
          /* @__PURE__ */ a("strong", { children: n.command }),
          /* @__PURE__ */ a("span", { className: `workbench-check-status ${n.status}`, children: n.status }),
          /* @__PURE__ */ a("pre", { children: n.output || "(no output pasted)" })
        ] }, `${n.capturedAt}-${r}`))
      ] }) : null
    ] }) : /* @__PURE__ */ a("pre", { className: "workbench-terminal-placeholder", children: "$ open the command palette and run Checks: Open Manual Check Panel" })
  ] });
}
function xl(e) {
  return /* @__PURE__ */ u("section", { className: "workbench-bottom-panel", "aria-label": "Panel", children: [
    /* @__PURE__ */ u("div", { className: "workbench-bottom-tabs", children: [
      /* @__PURE__ */ a("button", { className: e.tab === "output" ? "active" : "", onClick: () => e.setTab("output"), children: "OUTPUT" }),
      /* @__PURE__ */ a("button", { className: e.tab === "terminal" ? "active" : "", onClick: () => e.setTab("terminal"), children: "MANUAL CHECKS" }),
      /* @__PURE__ */ a("span", {}),
      /* @__PURE__ */ a("button", { title: "Close Panel", onClick: e.onClose, children: /* @__PURE__ */ a(qe, { size: 14 }) })
    ] }),
    /* @__PURE__ */ u("div", { className: "workbench-bottom-body", children: [
      e.tab === "terminal" && /* @__PURE__ */ a(
        kl,
        {
          session: e.manualCheckSession,
          commandInput: e.manualCheckCommandInput,
          setCommandInput: e.setManualCheckCommandInput,
          outputInput: e.manualCheckOutputInput,
          setOutputInput: e.setManualCheckOutputInput,
          selectedCommand: e.manualCheckSelectedCommand,
          setSelectedCommand: e.setManualCheckSelectedCommand,
          status: e.manualCheckStatus,
          setStatus: e.setManualCheckStatus,
          copyCommand: e.copyManualCheckCommand,
          addCommand: e.addManualCheckCommand,
          recordResult: e.recordManualCheckResult,
          askAgent: e.askAgentFromManualChecks
        }
      ),
      e.tab === "output" && /* @__PURE__ */ a(Er, { outputs: e.outputs, clearOutputs: e.clearOutputs, deleteArtifact: e.deleteArtifact, runAiSynthesis: e.runAiSynthesis, captureManualCheck: e.captureManualCheck, openOutputAsFile: e.openOutputAsFile, compact: !0 })
    ] })
  ] });
}
function Pl(e) {
  const t = Ke(), [n, r] = T(!1), [i, s] = T(""), [o, c] = T([]), m = Se(null), b = Se(null), p = Se(null), [v, g] = T({ top: 0, right: 0 }), S = ue(
    () => e.chatThreads.map((z) => ({
      kind: "atomek-thread",
      id: `thread:${z.id}`,
      title: z.title,
      lastActivityAt: z.lastMessageAt ?? z.updatedAt ?? 0,
      thread: z
    })),
    [e.chatThreads]
  ), M = ue(() => o.map((z) => {
    const I = e.chatTargets.find((ge) => ge.id === z.targetId);
    return I ? {
      kind: "pod-agent",
      id: `agent:${z.targetId}`,
      title: I.label,
      subtitle: z.preview || I.description || `${z.messageCount} messages`,
      lastActivityAt: z.lastActivityAt,
      targetId: z.targetId
    } : null;
  }).filter((z) => z !== null), [o, e.chatTargets]), E = ue(() => [...S, ...M].sort((z, I) => I.lastActivityAt - z.lastActivityAt), [S, M]), J = ue(() => e.selectedChatTarget.kind === "atomek-ai" && e.chatThread ? E.find((z) => z.kind === "atomek-thread" && z.thread.id === e.chatThread?.id) ?? null : e.selectedChatTarget.kind !== "atomek-ai" ? E.find((z) => z.kind === "pod-agent" && z.targetId === e.selectedChatTarget.id) ?? null : null, [E, e.selectedChatTarget, e.chatThread]), F = N((z, I) => I ? `${z.title} ${z.kind === "pod-agent" ? z.subtitle : ""}`.toLowerCase().includes(I) : !0, []), ae = i.trim().toLowerCase(), me = J ? F(J, ae) : !1, W = ue(() => E.filter((z) => z.id !== J?.id).filter((z) => F(z, ae)), [E, J, ae, F]), Z = N(() => {
    c(Fi());
  }, []), K = N((z) => {
    r(!1), z.kind === "atomek-thread" ? (e.selectedChatTarget.kind !== "atomek-ai" && e.selectChatTarget(Ze.id), e.selectThread(z.thread.id)) : e.selectChatTarget(z.targetId), e.tab !== "chat" && e.setTab("chat");
  }, [e]), V = N((z) => {
    window.confirm(`Delete conversation "${z.title}"?`) && (z.kind === "atomek-thread" ? e.deleteThread(z.thread.id) : (Di(z.targetId), Z()));
  }, [e, Z]);
  return oe(() => {
    if (!n) {
      s("");
      return;
    }
    Z();
    const z = () => {
      const ee = m.current;
      if (!ee) return;
      const fe = ee.getBoundingClientRect();
      g({
        top: fe.bottom + 6,
        right: Math.max(8, window.innerWidth - fe.right)
      });
    };
    z(), window.setTimeout(() => p.current?.focus(), 0);
    const I = (ee) => {
      const fe = ee.target;
      fe && (b.current?.contains(fe) || m.current?.contains(fe) || r(!1));
    }, ge = (ee) => {
      ee.key === "Escape" && r(!1);
    };
    return window.addEventListener("resize", z), window.addEventListener("scroll", z, !0), window.addEventListener("mousedown", I), window.addEventListener("keydown", ge), () => {
      window.removeEventListener("resize", z), window.removeEventListener("scroll", z, !0), window.removeEventListener("mousedown", I), window.removeEventListener("keydown", ge);
    };
  }, [n]), /* @__PURE__ */ u("aside", { className: "workbench-secondary", children: [
    /* @__PURE__ */ a("div", { className: "workbench-secondary-resizer", onPointerDown: e.onResizeStart, title: t("shell.resizeChat") }),
    /* @__PURE__ */ u("div", { className: "workbench-secondary-tabs", children: [
      /* @__PURE__ */ u("div", { className: "workbench-secondary-tab-group", children: [
        /* @__PURE__ */ a("button", { className: `workbench-secondary-tab ${e.tab === "chat" ? "active" : ""}`, onClick: () => e.setTab("chat"), children: t("secondary.chat") }),
        /* @__PURE__ */ a("button", { className: `workbench-secondary-tab ${e.tab === "agents" ? "active" : ""}`, onClick: () => e.setTab("agents"), children: t("secondary.agents") }),
        /* @__PURE__ */ a("button", { className: `workbench-secondary-tab ${e.tab === "outputs" ? "active" : ""}`, onClick: () => e.setTab("outputs"), children: t("secondary.outputs") })
      ] }),
      /* @__PURE__ */ u("div", { className: "workbench-secondary-actions", children: [
        /* @__PURE__ */ a("button", { title: t("secondary.newChat"), onClick: e.newChat, children: /* @__PURE__ */ a(Fn, { size: 15 }) }),
        /* @__PURE__ */ a(
          "button",
          {
            ref: m,
            title: t("secondary.pastConversations"),
            "aria-label": t("secondary.pastConversations"),
            "aria-expanded": n,
            onClick: () => {
              Z(), r((z) => !z);
            },
            className: n ? "is-active" : "",
            children: /* @__PURE__ */ a(Ya, { size: 15 })
          }
        ),
        /* @__PURE__ */ a("button", { title: t("secondary.chatSettings"), onClick: e.openSettings, children: /* @__PURE__ */ a(Ja, { size: 16 }) }),
        /* @__PURE__ */ a("button", { title: t("secondary.closeChat"), onClick: e.onClose, children: /* @__PURE__ */ a(qe, { size: 15 }) })
      ] })
    ] }),
    n && typeof document < "u" ? Ps(
      /* @__PURE__ */ u(
        "div",
        {
          ref: b,
          className: "workbench-history-portal",
          style: { top: v.top, right: v.right },
          role: "menu",
          children: [
            /* @__PURE__ */ u("div", { className: "workbench-history-portal-search", children: [
              /* @__PURE__ */ a(Ua, { size: 14 }),
              /* @__PURE__ */ a(
                "input",
                {
                  ref: p,
                  type: "text",
                  placeholder: t("history.search"),
                  value: i,
                  onChange: (z) => s(z.target.value),
                  onKeyDown: (z) => {
                    if (z.key === "Enter") {
                      const I = me && J ? J : W[0];
                      I && K(I);
                    }
                  }
                }
              )
            ] }),
            /* @__PURE__ */ u("div", { className: "workbench-history-portal-list", children: [
              J && me ? /* @__PURE__ */ u(De, { children: [
                /* @__PURE__ */ a("div", { className: "workbench-history-portal-group", children: t("history.current") }),
                /* @__PURE__ */ u(
                  "div",
                  {
                    className: "workbench-history-portal-item active",
                    role: "menuitem",
                    onClick: () => K(J),
                    title: J.title,
                    children: [
                      /* @__PURE__ */ u("span", { className: "workbench-history-portal-title", children: [
                        J.kind === "pod-agent" ? /* @__PURE__ */ a("span", { className: "workbench-history-portal-badge", children: t("history.agent") }) : null,
                        J.title
                      ] }),
                      /* @__PURE__ */ a("small", { children: Kn(J.lastActivityAt) }),
                      /* @__PURE__ */ a(
                        "button",
                        {
                          className: "workbench-history-portal-delete",
                          onClick: (z) => {
                            z.stopPropagation(), V(J);
                          },
                          title: t("history.deleteConversation"),
                          "aria-label": t("history.deleteConversation"),
                          children: /* @__PURE__ */ a(xa, { size: 13 })
                        }
                      )
                    ]
                  }
                )
              ] }) : null,
              W.length > 0 ? /* @__PURE__ */ u(De, { children: [
                /* @__PURE__ */ a("div", { className: "workbench-history-portal-group", children: t("history.recent") }),
                W.map((z) => /* @__PURE__ */ u(
                  "div",
                  {
                    className: "workbench-history-portal-item",
                    role: "menuitem",
                    onClick: () => K(z),
                    title: z.title,
                    children: [
                      /* @__PURE__ */ u("span", { className: "workbench-history-portal-title", children: [
                        z.kind === "pod-agent" ? /* @__PURE__ */ a("span", { className: "workbench-history-portal-badge", children: t("history.agent") }) : null,
                        z.title
                      ] }),
                      /* @__PURE__ */ a("small", { children: Kn(z.lastActivityAt) }),
                      /* @__PURE__ */ a(
                        "button",
                        {
                          className: "workbench-history-portal-delete",
                          onClick: (I) => {
                            I.stopPropagation(), V(z);
                          },
                          title: t("history.deleteConversation"),
                          "aria-label": t("history.deleteConversation"),
                          children: /* @__PURE__ */ a(xa, { size: 13 })
                        }
                      )
                    ]
                  },
                  z.id
                ))
              ] }) : null,
              !me && W.length === 0 ? /* @__PURE__ */ a("div", { className: "workbench-history-portal-empty", children: i.trim() ? t("history.noMatches") : t("history.empty") }) : null
            ] }),
            /* @__PURE__ */ u("div", { className: "workbench-history-portal-footer", children: [
              /* @__PURE__ */ u("span", { children: [
                /* @__PURE__ */ a("kbd", { children: "↩" }),
                " ",
                t("history.open")
              ] }),
              /* @__PURE__ */ u("span", { children: [
                /* @__PURE__ */ a("kbd", { children: "Esc" }),
                " ",
                t("history.close")
              ] })
            ] })
          ]
        }
      ),
      document.body
    ) : null,
    e.tab === "chat" ? /* @__PURE__ */ a(Nl, { ...e }) : e.tab === "agents" ? /* @__PURE__ */ a(
      qr,
      {
        host: e.host,
        setStatus: e.setStatus,
        attachSkillToChat: e.attachSkillToChat,
        saveLocalJobOutput: e.saveLocalJobOutput,
        activeFile: e.activeFile,
        openEditors: e.openEditors,
        variant: "dock"
      }
    ) : /* @__PURE__ */ a(Er, { outputs: e.outputs, clearOutputs: e.clearOutputs, deleteArtifact: e.deleteArtifact, runAiSynthesis: e.runAiSynthesis, captureManualCheck: e.captureManualCheck, openOutputAsFile: e.openOutputAsFile, previewEditFromOutput: e.previewEditFromOutput, canPreviewEdit: e.canPreviewEdit })
  ] });
}
function Dr({ body: e }) {
  const t = ue(() => Gc(e), [e]), [n, r] = T(null), i = N((s, o) => {
    (async () => await gt(o) && (r(s), window.setTimeout(() => r((m) => m === s ? null : m), 1200)))();
  }, []);
  return /* @__PURE__ */ a("div", { className: "workbench-rich-body", children: t.map((s) => {
    if (s.type === "code") {
      const o = n === s.key;
      return /* @__PURE__ */ u("div", { className: "workbench-rich-code", children: [
        /* @__PURE__ */ u("div", { className: "workbench-rich-code-head", children: [
          /* @__PURE__ */ a("span", { children: s.language }),
          /* @__PURE__ */ u("button", { onClick: () => i(s.key, s.body), title: "Copy code block", children: [
            o ? /* @__PURE__ */ a(Ms, { size: 12 }) : /* @__PURE__ */ a(In, { size: 12 }),
            o ? "Copied" : "Copy"
          ] })
        ] }),
        /* @__PURE__ */ a("pre", { children: /* @__PURE__ */ a("code", { children: s.body }) })
      ] }, s.key);
    }
    return /* @__PURE__ */ a(
      "div",
      {
        className: "workbench-rich-markdown",
        dangerouslySetInnerHTML: { __html: nr(s.body) }
      },
      s.key
    );
  }) });
}
function Nl(e) {
  const t = Ke(), n = e.selectedChatTarget.available, r = e.selectedChatTarget.kind === "atomek-ai", i = e.chatInput.trim().length > 0 && !e.busy && n, s = Se(null), [o, c] = T(!0), [m, b] = T(!1), p = Se(null), v = Se(null), g = Se(null), S = Se(null), [M, E] = T(!1), [J, F] = T(0), ae = typeof window < "u" && !!(window.SpeechRecognition || window.webkitSpeechRecognition), me = N(() => {
    const f = g.current;
    try {
      f?.stop?.();
    } catch {
    }
    g.current = null, S.current !== null && (window.clearInterval(S.current), S.current = null), E(!1), F(0);
  }, []), W = N(() => {
    if (!ae || M) return;
    const f = window.SpeechRecognition || window.webkitSpeechRecognition;
    if (!f) return;
    const q = new f();
    q.lang = navigator.language || "en-US", q.interimResults = !1, q.continuous = !1, q.onresult = (G) => {
      let k = "";
      for (let O = G.resultIndex; O < G.results.length; O += 1) {
        const x = G.results[O];
        x.isFinal && (k += x[0].transcript);
      }
      if (k) {
        const O = e.chatInput.trim().length > 0 ? `${e.chatInput} ${k}` : k;
        e.setChatInput(O);
      }
    }, q.onerror = () => {
      me();
    }, q.onend = () => {
      me();
    }, g.current = q;
    try {
      q.start(), E(!0), F(0), S.current = window.setInterval(() => F((G) => G + 1), 1e3);
    } catch {
      me();
    }
  }, [ae, M, e, me]);
  oe(() => () => {
    S.current !== null && window.clearInterval(S.current);
    const f = g.current;
    try {
      f?.stop?.();
    } catch {
    }
  }, []);
  const Z = (f) => {
    const q = Math.floor(f / 60), G = f % 60;
    return `${q.toString().padStart(1, "0")}:${G.toString().padStart(2, "0")}`;
  }, K = () => p.current?.removeAttribute("open"), V = () => v.current?.removeAttribute("open"), z = (f) => {
    e.setContextScope(f), f === "indexed-project" && e.refreshProjectIndex(), K();
  }, I = (f) => {
    e.selectChatTarget(f), V();
  }, ge = ue(() => e.chatMessages.map((f) => `${f.id}:${f.status ?? ""}:${f.body.length}`).join("|"), [e.chatMessages]);
  oe(() => {
    const f = s.current;
    f && (o ? (f.scrollTop = f.scrollHeight, b(!1)) : b(!0));
  }, [o, ge]);
  const ee = N(() => {
    const f = s.current;
    if (!f) return;
    const G = f.scrollHeight - f.scrollTop - f.clientHeight < 48;
    c(G), G && b(!1);
  }, []), fe = N(() => {
    const f = s.current;
    f && (f.scrollTop = f.scrollHeight, c(!0), b(!1));
  }, []), Te = N((f) => {
    gt(f.body);
  }, []), He = Se(null), xe = () => He.current?.removeAttribute("open");
  return /* @__PURE__ */ u("div", { className: "workbench-chat-wrap", children: [
    /* @__PURE__ */ u("div", { className: "workbench-chat-threadbar", children: [
      /* @__PURE__ */ a("span", { className: "workbench-chat-thread-title", title: e.chatThread?.title ?? e.selectedChatTarget.label, children: r ? e.chatThread?.title ?? t("chat.atomekChat") : t("chat.session", { name: e.selectedChatTarget.label }) }),
      /* @__PURE__ */ u("span", { className: "workbench-chat-thread-actions", children: [
        /* @__PURE__ */ a(
          "button",
          {
            className: "workbench-chat-iconbtn",
            onClick: () => {
              e.newChat();
            },
            disabled: e.busy,
            title: t(r ? "chat.newConversation" : "chat.clearConversation"),
            "aria-label": t("chat.newConversation"),
            children: /* @__PURE__ */ a(Fn, { size: 15 })
          }
        ),
        r && e.chatThreads.length > 0 ? /* @__PURE__ */ u("details", { ref: He, className: "workbench-chat-iconmenu", children: [
          /* @__PURE__ */ a("summary", { className: "workbench-chat-iconbtn", title: t("chat.history"), "aria-label": t("chat.history"), children: /* @__PURE__ */ a(Ya, { size: 15 }) }),
          /* @__PURE__ */ u("div", { className: "workbench-chat-history-pop", role: "menu", children: [
            /* @__PURE__ */ a("div", { className: "workbench-chat-history-header", children: t("chat.chats", { count: e.chatThreads.length }) }),
            e.chatThreads.map((f) => /* @__PURE__ */ u(
              "button",
              {
                className: f.id === e.chatThread?.id ? "active" : "",
                onClick: () => {
                  e.selectThread(f.id), xe();
                },
                title: f.title,
                children: [
                  /* @__PURE__ */ a("span", { className: "workbench-chat-history-title", children: f.title }),
                  /* @__PURE__ */ a("small", { children: Kn(f.lastMessageAt ?? f.updatedAt) })
                ]
              },
              f.id
            ))
          ] })
        ] }) : null,
        r ? /* @__PURE__ */ u("details", { className: "workbench-chat-iconmenu", children: [
          /* @__PURE__ */ a("summary", { className: "workbench-chat-iconbtn", title: t("chat.actions"), "aria-label": t("chat.actions"), children: /* @__PURE__ */ a(Ja, { size: 15 }) }),
          /* @__PURE__ */ u("div", { className: "workbench-chat-threadmenu-pop", children: [
            /* @__PURE__ */ a(
              "button",
              {
                onClick: (f) => {
                  if (f.currentTarget.closest("details")?.removeAttribute("open"), !e.chatThread) return;
                  const q = window.prompt(t("chat.renamePrompt"), e.chatThread.title);
                  q !== null && e.renameThread(e.chatThread.id, q);
                },
                disabled: !e.chatThread || e.busy,
                children: "Rename"
              }
            ),
            /* @__PURE__ */ a(
              "button",
              {
                className: "danger",
                onClick: (f) => {
                  f.currentTarget.closest("details")?.removeAttribute("open"), e.chatThread && window.confirm(`Delete chat "${e.chatThread.title}"?`) && e.deleteThread(e.chatThread.id);
                },
                disabled: !e.chatThread || e.busy,
                children: "Delete"
              }
            )
          ] })
        ] }) : null
      ] })
    ] }),
    /* @__PURE__ */ u("div", { ref: s, className: "workbench-chat-transcript", onScroll: ee, children: [
      e.chatMessages.length === 0 ? /* @__PURE__ */ a("div", { className: "workbench-chat-empty", children: /* @__PURE__ */ u("div", { children: [
        /* @__PURE__ */ a(Gs, { size: 48 }),
        /* @__PURE__ */ a("h3", { children: t("chat.buildWith", { name: e.selectedChatTarget.label }) }),
        /* @__PURE__ */ a("p", { children: e.selectedChatTarget.kind === "atomek-ai" ? t("chat.emptyAtomek") : t("chat.emptyPod") }),
        /* @__PURE__ */ a("p", { className: "workbench-chat-empty-link", children: e.aiStatus.available ? e.aiStatus.label : e.aiStatus.reason ?? e.aiStatus.label })
      ] }) }) : e.chatMessages.map((f) => /* @__PURE__ */ u("div", { className: `workbench-chat-message ${f.role}`, children: [
        /* @__PURE__ */ a("strong", { children: f.role === "user" ? t("chat.you") : f.sourceLabel ?? "Atomek" }),
        f.status === "streaming" ? /* @__PURE__ */ u("em", { children: [
          " ",
          t("chat.streaming")
        ] }) : null,
        f.status === "error" ? /* @__PURE__ */ u("em", { children: [
          " ",
          t("chat.error")
        ] }) : null,
        /* @__PURE__ */ a("br", {}),
        /* @__PURE__ */ a(Dr, { body: f.body }),
        f.gatewayLabel ? /* @__PURE__ */ u(De, { children: [
          /* @__PURE__ */ a("br", {}),
          /* @__PURE__ */ a("small", { children: f.gatewayLabel })
        ] }) : null,
        f.role === "assistant" && f.status !== "streaming" && f.status !== "error" ? /* @__PURE__ */ u("div", { className: "workbench-chat-message-actions", children: [
          /* @__PURE__ */ a("button", { className: "workbench-chat-message-action", onClick: () => Te(f), title: t("chat.copyAnswer"), "aria-label": t("chat.copyAnswer"), children: /* @__PURE__ */ a(In, { size: 14 }) }),
          /* @__PURE__ */ a("button", { className: "workbench-chat-message-action", onClick: () => e.saveMessageAsArtifact(f), title: t("chat.saveArtifact"), "aria-label": t("chat.saveArtifact"), children: /* @__PURE__ */ a(Qn, { size: 14 }) }),
          /* @__PURE__ */ a("button", { className: "workbench-chat-message-action", onClick: () => e.rememberMessage(f), title: "Store in Atomek memory", "aria-label": t("chat.remember"), children: /* @__PURE__ */ a(Us, { size: 14 }) }),
          /* @__PURE__ */ a("button", { className: "workbench-chat-message-action", onClick: () => e.previewEditFromMessage(f), disabled: e.workspaceFileCount === 0, title: t("chat.previewPatch"), "aria-label": t("chat.previewPatch"), children: /* @__PURE__ */ a(Bn, { size: 14 }) }),
          /* @__PURE__ */ a("button", { className: "workbench-chat-message-action regen", onClick: () => e.regenerateMessage(f), disabled: e.busy, title: t("chat.regenerate"), "aria-label": t("chat.regenerate"), children: /* @__PURE__ */ a(bt, { size: 14 }) })
        ] }) : null,
        f.role === "assistant" && f.status === "error" ? /* @__PURE__ */ u("div", { className: "workbench-chat-message-actions", children: [
          /* @__PURE__ */ a("button", { className: "workbench-chat-message-action", onClick: () => Te(f), title: t("chat.copyError"), "aria-label": t("chat.copyError"), children: /* @__PURE__ */ a(In, { size: 14 }) }),
          /* @__PURE__ */ a("button", { className: "workbench-chat-message-action regen", onClick: () => e.regenerateMessage(f), disabled: e.busy, title: t("chat.retry"), "aria-label": t("chat.retry"), children: /* @__PURE__ */ a(bt, { size: 14 }) })
        ] }) : null
      ] }, f.id)),
      m ? /* @__PURE__ */ a("button", { className: "workbench-chat-jump", onClick: fe, children: t("chat.jumpLatest") }) : null
    ] }),
    /* @__PURE__ */ u("div", { className: "workbench-chat-composer", children: [
      /* @__PURE__ */ u("div", { className: "workbench-chat-tip", children: [
        /* @__PURE__ */ a("span", { children: t("chat.target") }),
        /* @__PURE__ */ a("strong", { children: e.selectedChatTarget.label }),
        /* @__PURE__ */ a("span", { className: "workbench-chat-tip-sep", children: "·" }),
        /* @__PURE__ */ a("span", { children: t("chat.context") }),
        /* @__PURE__ */ a("strong", { children: Bi(e.contextScope) }),
        /* @__PURE__ */ a("em", { children: e.selectedChatTarget.kind === "atomek-ai" ? Fa(e.chatSettings, e.aiStatus.label, e.memoryHitCount) : e.selectedChatTarget.description })
      ] }),
      /* @__PURE__ */ u("div", { className: "workbench-chat-box", children: [
        /* @__PURE__ */ u("div", { className: "workbench-chat-attachments", children: [
          /* @__PURE__ */ u(
            "select",
            {
              className: "workbench-chat-context-select",
              value: e.contextScope,
              onChange: (f) => e.setContextScope(f.target.value),
              disabled: e.busy,
              title: "Context scope for next message",
              children: [
                /* @__PURE__ */ a("option", { value: "none", children: t("chat.noContext") }),
                /* @__PURE__ */ a("option", { value: "active-selection", children: t("chat.selection") }),
                /* @__PURE__ */ a("option", { value: "active-file", children: t("chat.activeFile") }),
                /* @__PURE__ */ a("option", { value: "open-editors", children: t("chat.openEditors") }),
                /* @__PURE__ */ a("option", { value: "indexed-project", children: t("chat.indexedProject") })
              ]
            }
          ),
          e.contextScope === "indexed-project" ? /* @__PURE__ */ u(De, { children: [
            /* @__PURE__ */ u("button", { className: "workbench-chat-chip-button", onClick: e.refreshProjectIndex, disabled: e.busy, children: [
              /* @__PURE__ */ a(bt, { size: 12 }),
              " ",
              t("chat.index")
            ] }),
            /* @__PURE__ */ u("span", { className: `workbench-chat-chip ${e.projectIndexStale ? "warn" : "muted"}`, title: "Project index used for query-scoped retrieval", children: [
              /* @__PURE__ */ a(on, { size: 13 }),
              " ",
              e.projectIndexSummary,
              e.projectIndexStale ? ` · ${t("chat.stale")}` : ""
            ] })
          ] }) : null,
          e.contextAttachments.length === 0 ? /* @__PURE__ */ u("span", { className: "workbench-chat-chip muted", children: [
            /* @__PURE__ */ a(ka, { size: 13 }),
            " ",
            t("chat.noFileContext")
          ] }) : e.contextAttachments.map((f) => {
            const q = typeof f.score == "number" ? f.score.toFixed(2) : null, G = typeof f.vectorScore == "number" ? f.vectorScore.toFixed(2) : null, k = typeof f.keywordScore == "number" ? f.keywordScore.toFixed(1) : null, O = [
              f.path,
              f.range ? `${f.range.startLineNumber}:${f.range.startColumn}-${f.range.endLineNumber}:${f.range.endColumn}` : null,
              q ? `score ${q}` : null,
              G ? `vector ${G}` : null,
              k ? `keyword ${k}` : null,
              f.snippet,
              f.dirty ? "dirty" : null
            ].filter(Boolean).join(" · ");
            return /* @__PURE__ */ u("span", { className: "workbench-chat-chip", title: O, children: [
              /* @__PURE__ */ a("button", { className: "workbench-chat-chip-open", onClick: () => e.revealContextAttachment(f), disabled: !f.fileId, title: t("chat.revealContext"), children: /* @__PURE__ */ a(ka, { size: 13 }) }),
              f.label,
              q ? /* @__PURE__ */ a("small", { children: q }) : null,
              f.snippet ? /* @__PURE__ */ u("small", { children: [
                f.snippet.slice(0, 60),
                f.snippet.length > 60 ? "…" : ""
              ] }) : null,
              f.dirty ? /* @__PURE__ */ a("small", { children: t("chat.dirty") }) : null,
              f.removable ? /* @__PURE__ */ a("button", { className: "workbench-chat-chip-remove", onClick: () => e.removeContextAttachment(f), title: t("chat.removeContext"), children: /* @__PURE__ */ a(qe, { size: 11 }) }) : null
            ] }, f.id);
          }),
          /* @__PURE__ */ a("button", { className: "workbench-chat-chip-button", onClick: () => e.runQuickPrompt("explain"), disabled: !e.activeFile || e.busy, children: t("chat.explain") }),
          /* @__PURE__ */ a("button", { className: "workbench-chat-chip-button", onClick: () => e.runQuickPrompt("improve"), disabled: !e.activeFile || e.busy, children: t("chat.improve") }),
          /* @__PURE__ */ a("button", { className: "workbench-chat-chip-button", onClick: () => e.runQuickPrompt("edit"), disabled: !e.activeFile || e.busy, children: t("chat.edit") }),
          /* @__PURE__ */ a("button", { className: "workbench-chat-chip-button", onClick: () => e.runQuickPrompt("draft"), disabled: e.busy, children: t("chat.draft") })
        ] }),
        e.pendingPatchPrompt ? /* @__PURE__ */ a("button", { className: "workbench-chat-generate-patch", onClick: e.generatePatchPrompt, disabled: e.busy, children: t("chat.generatePatch") }) : null,
        /* @__PURE__ */ a(
          "textarea",
          {
            className: "workbench-chat-textarea",
            value: e.chatInput,
            onChange: (f) => e.setChatInput(f.target.value),
            onKeyDown: (f) => {
              f.key === "Enter" && !f.shiftKey && (f.preventDefault(), e.busy || e.askAgent());
            },
            placeholder: e.selectedChatTarget.kind === "atomek-ai" ? t("chat.askAtomek") : t("chat.askTarget", { name: e.selectedChatTarget.label }),
            rows: 3
          }
        ),
        /* @__PURE__ */ u("div", { className: "workbench-chat-toolbar atomek-input", children: [
          /* @__PURE__ */ u("details", { ref: p, className: "workbench-chat-attach", children: [
            /* @__PURE__ */ a("summary", { title: t("chat.addContext"), "aria-label": t("chat.addContext"), children: /* @__PURE__ */ a(Fn, { size: 16 }) }),
            /* @__PURE__ */ u("div", { className: "workbench-chat-attach-menu", role: "menu", children: [
              /* @__PURE__ */ a("button", { onClick: () => z("active-file"), disabled: !e.activeFile, title: t("chat.useActiveFile"), children: t("chat.activeFile") }),
              /* @__PURE__ */ a("button", { onClick: () => z("active-selection"), disabled: !e.activeFile, title: t("chat.useSelection"), children: t("chat.selection") }),
              /* @__PURE__ */ a("button", { onClick: () => z("open-editors"), title: t("chat.useOpenEditors"), children: t("chat.openEditors") }),
              /* @__PURE__ */ a("button", { onClick: () => z("indexed-project"), title: t("chat.useIndexedProject"), children: t("chat.indexedProject") }),
              /* @__PURE__ */ a("button", { onClick: () => z("none"), title: t("chat.noContext"), children: t("chat.noContext") })
            ] })
          ] }),
          M ? /* @__PURE__ */ u("div", { className: "workbench-chat-recording", children: [
            /* @__PURE__ */ a("button", { className: "workbench-chat-recording-cancel", onClick: me, title: t("chat.cancelRecording"), "aria-label": t("chat.cancelRecording"), children: /* @__PURE__ */ a(qe, { size: 14 }) }),
            /* @__PURE__ */ a("span", { className: "workbench-chat-recording-wave", "aria-hidden": "true", children: Array.from({ length: 14 }).map((f, q) => /* @__PURE__ */ a("span", { style: { animationDelay: `${q * 0.05}s` } }, q)) }),
            /* @__PURE__ */ a("span", { className: "workbench-chat-recording-dot", "aria-hidden": "true" }),
            /* @__PURE__ */ a("span", { className: "workbench-chat-recording-time", children: Z(J) })
          ] }) : /* @__PURE__ */ a("span", { className: "workbench-chat-route-summary", children: e.selectedChatTarget.kind === "atomek-ai" ? Fa(e.chatSettings, e.aiStatus.label, e.memoryHitCount) : e.selectedChatTarget.description }),
          /* @__PURE__ */ u("div", { className: "workbench-chat-toolbar-right", children: [
            M ? null : /* @__PURE__ */ a(
              "button",
              {
                className: "workbench-chat-mic",
                onClick: W,
                disabled: !ae || e.busy,
                title: t(ae ? "chat.voiceInput" : "chat.voiceUnsupported"),
                "aria-label": t("chat.voiceInput"),
                children: /* @__PURE__ */ a($s, { size: 16 })
              }
            ),
            /* @__PURE__ */ u("details", { ref: v, className: "workbench-chat-target", children: [
              /* @__PURE__ */ u("summary", { title: t("chat.chooseTarget"), "aria-label": t("chat.chooseTarget"), children: [
                /* @__PURE__ */ a("span", { className: "workbench-chat-target-label", children: e.selectedChatTarget.label }),
                /* @__PURE__ */ a(Xt, { size: 14 })
              ] }),
              /* @__PURE__ */ a("div", { className: "workbench-chat-target-menu", role: "menu", children: e.chatTargets.map((f) => /* @__PURE__ */ u(
                "button",
                {
                  onClick: () => I(f.id),
                  disabled: !f.available,
                  className: f.id === e.selectedChatTarget.id ? "active" : "",
                  title: f.description,
                  children: [
                    /* @__PURE__ */ a("span", { className: "workbench-chat-target-row-label", children: f.label }),
                    f.kind === "pod-agent" && f.status !== "running" ? /* @__PURE__ */ a("small", { children: f.status }) : null
                  ]
                },
                f.id
              )) })
            ] }),
            e.busy ? /* @__PURE__ */ a("button", { className: "workbench-chat-send stop", onClick: e.stopChat, title: t("chat.stop"), "aria-label": t("chat.stop"), children: /* @__PURE__ */ a(Ga, { size: 14 }) }) : /* @__PURE__ */ a("button", { className: `workbench-chat-send ${i ? "ready" : ""}`, onClick: e.askAgent, title: t("chat.send"), disabled: !i, "aria-label": t("chat.sendMessage"), children: /* @__PURE__ */ a(Cs, { size: 18 }) })
          ] })
        ] })
      ] })
    ] })
  ] });
}
function Rr(e) {
  return e === "remote" ? "Remote AIL" : e === "local" ? "Local AIL" : "Auto";
}
function Fa(e, t, n) {
  e.model;
  const i = [e.gatewayPreference === "auto" ? t : Rr(e.gatewayPreference)];
  return n > 0 && i.push(`${n} memories`), i.join(" · ");
}
function Kn(e) {
  return !Number.isFinite(e) || e <= 0 ? "new" : new Date(e).toLocaleString(void 0, { month: "short", day: "numeric", hour: "2-digit", minute: "2-digit" });
}
function Da(e) {
  return /```(?:diff|patch)\b/i.test(e) || /^diff --git /m.test(e) || /^--- .+\n\+\+\+ /m.test(e) || /```[\w.+-]*\s*\n[\s\S]{80,}```/.test(e);
}
function Tl(e) {
  return /\b(change|edit|modify|replace|update|rename|fix|rewrite|apply)\b/i.test(e) && /\b(file|code|author|title|line|function|component|content|text|this|it)\b/i.test(e);
}
function zl(e) {
  return [
    e,
    "Atomek edit instruction: if this request should change an open file, return an applicable git-style unified diff in a fenced diff block. Use paths exactly as shown in the attached context. If one whole-file replacement is safer, return a fenced atomek-replace block. Do not claim a file changed unless you provide a patch/replacement Atomek can preview."
  ].join(`

`);
}
function Al({ host: e, appName: t, currentVersion: n }) {
  const r = e.apps, [i, s] = T(null), [o, c] = T(!1), [m, b] = T(null), p = N(async () => {
    if (!r?.checkUpdate) {
      b("Update checks need a newer Tytus OS build.");
      return;
    }
    c(!0), b(null);
    try {
      const M = await r.checkUpdate();
      s(M), M.error && b(M.error);
    } catch (M) {
      b(M instanceof Error ? M.message : String(M));
    } finally {
      c(!1);
    }
  }, [r]);
  oe(() => {
    p();
  }, [p]);
  const v = N(async () => {
    if (!r?.updateSelf) {
      b("Update needs a newer Tytus OS build.");
      return;
    }
    c(!0), b(null);
    try {
      const M = await r.updateSelf();
      s(M), b(M.error ?? `${t} updated. Close and reopen the app to load the new bundle.`);
    } catch (M) {
      b(M instanceof Error ? M.message : String(M));
    } finally {
      c(!1);
    }
  }, [r, t]), g = i?.latestVersion ?? n, S = !!i?.updateAvailable;
  return /* @__PURE__ */ u("div", { className: "workbench-settings-section", children: [
    /* @__PURE__ */ a("h3", { children: "App version" }),
    /* @__PURE__ */ u("p", { children: [
      "Installed v",
      i?.currentVersion ?? n,
      ". Latest available v",
      g,
      "."
    ] }),
    /* @__PURE__ */ u("div", { style: { display: "flex", gap: 8, flexWrap: "wrap" }, children: [
      /* @__PURE__ */ u("button", { className: "workbench-button-subtle", onClick: () => void p(), disabled: o, children: [
        /* @__PURE__ */ a(bt, { size: 14 }),
        o ? "Checking…" : "Check for update"
      ] }),
      S ? /* @__PURE__ */ a("button", { className: "workbench-button-primary", onClick: () => void v(), disabled: o, children: o ? "Updating…" : `Update ${t}` }) : null
    ] }),
    /* @__PURE__ */ a("div", { className: "workbench-settings-note", children: m ?? (i ? S ? "Update available." : "You are running the latest available version." : "Checking latest version…") })
  ] });
}
function Ll(e) {
  const [t, n] = T([]), [r, i] = T("Loading gateway models…"), [s, o] = T([]), [c, m] = T("Checking embedding capability…");
  oe(() => {
    const g = new AbortController();
    return (async () => {
      if (!e.host.ai?.listModels) {
        n([]), i("This Tytus build does not expose model discovery yet.");
        return;
      }
      i("Loading gateway models…");
      try {
        const M = await e.host.ai.listModels({
          gatewayPreference: e.chatSettings.gatewayPreference,
          signal: g.signal
        });
        if (g.signal.aborted) return;
        n(M.map((E) => ({ id: E.id, gatewayLabel: E.gatewayLabel }))), i(M.length > 0 ? `${M.length} models discovered from AIL.` : "No models discovered. You can still enter any AIL alias manually.");
      } catch (M) {
        if (g.signal.aborted) return;
        n([]), i(`Model discovery failed: ${M instanceof Error ? M.message : String(M)}`);
      }
    })(), () => g.abort();
  }, [e.chatSettings.gatewayPreference, e.host.ai]), oe(() => {
    const g = new AbortController();
    return (async () => {
      const M = fc(e.host);
      if (M) {
        o([]), m(M);
        return;
      }
      m("Loading embedding-capable models from AIL…");
      try {
        const E = await mc(e.host, {
          gatewayPreference: e.chatSettings.gatewayPreference,
          signal: g.signal
        });
        if (g.signal.aborted) return;
        o(E.map((J) => ({ id: J.id, gatewayLabel: J.gatewayLabel ?? J.source ?? "AIL" }))), m(E.length > 0 ? `${E.length} embedding models discovered from AIL metadata.` : "AIL embedding API is present, but no embedding-capable model metadata was returned.");
      } catch (E) {
        if (g.signal.aborted) return;
        o([]), m(`Embedding model discovery failed: ${E instanceof Error ? E.message : String(E)}`);
      }
    })(), () => g.abort();
  }, [e.chatSettings.gatewayPreference, e.host]);
  const b = (g) => {
    e.onChange({ ...e.chatSettings, gatewayPreference: g });
  }, p = (g) => {
    e.onChange({ ...e.chatSettings, model: g });
  }, v = (g) => {
    e.onChange({ ...e.chatSettings, embeddingModel: g });
  };
  return /* @__PURE__ */ a("div", { className: "workbench-settings-tab", children: /* @__PURE__ */ u("section", { className: "workbench-settings-page", "aria-label": "Atomek Settings", children: [
    /* @__PURE__ */ u("header", { className: "workbench-settings-header", children: [
      /* @__PURE__ */ a(Ba, { size: 15 }),
      /* @__PURE__ */ a("strong", { children: "Atomek Settings" }),
      /* @__PURE__ */ u("span", { style: { fontSize: 11, color: "var(--accent-primary)", border: "1px solid color-mix(in srgb, var(--accent-primary) 40%, transparent)", borderRadius: 999, padding: "2px 7px", fontWeight: 800 }, children: [
        "v",
        Va
      ] }),
      /* @__PURE__ */ a("button", { onClick: e.onClose, title: "Close", children: /* @__PURE__ */ a(qe, { size: 15 }) })
    ] }),
    /* @__PURE__ */ u("div", { className: "workbench-settings-body", children: [
      /* @__PURE__ */ a(Al, { host: e.host, appName: "Atomek", currentVersion: Va }),
      /* @__PURE__ */ u("div", { className: "workbench-settings-section", children: [
        /* @__PURE__ */ a("h3", { children: "Chat AI routing" }),
        /* @__PURE__ */ a("p", { children: "Choose which AIL gateway Atomek uses. Model names are not hardcoded here: enter an AIL alias/model from your global gateway config, or leave it empty for the gateway default." }),
        /* @__PURE__ */ u("label", { className: "workbench-settings-label", children: [
          "Gateway",
          /* @__PURE__ */ u(
            "select",
            {
              value: e.chatSettings.gatewayPreference,
              onChange: (g) => b(g.target.value),
              children: [
                /* @__PURE__ */ a("option", { value: "auto", children: "Auto failover" }),
                /* @__PURE__ */ a("option", { value: "remote", children: "Remote Tytus AIL only" }),
                /* @__PURE__ */ a("option", { value: "local", children: "Local AIL only" })
              ]
            }
          )
        ] }),
        /* @__PURE__ */ u("label", { className: "workbench-settings-label", children: [
          "Chat model alias",
          /* @__PURE__ */ a(
            "input",
            {
              value: e.chatSettings.model,
              onChange: (g) => p(g.target.value),
              list: "atomek-chat-models",
              placeholder: "Empty = AIL default/global alias",
              spellCheck: !1
            }
          ),
          /* @__PURE__ */ a("datalist", { id: "atomek-chat-models", children: t.map((g) => /* @__PURE__ */ a("option", { value: g.id, children: g.gatewayLabel }, `${g.gatewayLabel}:${g.id}`)) })
        ] }),
        /* @__PURE__ */ u("div", { className: "workbench-settings-note", children: [
          "Current request: ",
          Rr(e.chatSettings.gatewayPreference),
          e.chatSettings.model.trim() ? ` · ${e.chatSettings.model.trim()}` : " · gateway default"
        ] }),
        /* @__PURE__ */ a("div", { className: "workbench-settings-note", children: r })
      ] }),
      /* @__PURE__ */ u("div", { className: "workbench-settings-section", children: [
        /* @__PURE__ */ a("h3", { children: "Project context / embeddings" }),
        /* @__PURE__ */ a("p", { children: "Atomek keeps retrieval model selection dynamic. Leave empty for AIL global defaults, or pin an AIL embedding alias exposed by your gateway." }),
        /* @__PURE__ */ u("label", { className: "workbench-settings-label", children: [
          "Embedding model alias",
          /* @__PURE__ */ a(
            "input",
            {
              value: e.chatSettings.embeddingModel,
              onChange: (g) => v(g.target.value),
              list: "atomek-embedding-models",
              placeholder: "Empty = AIL embedding default/global alias",
              spellCheck: !1
            }
          ),
          /* @__PURE__ */ a("datalist", { id: "atomek-embedding-models", children: s.map((g) => /* @__PURE__ */ a("option", { value: g.id, children: g.gatewayLabel }, `${g.gatewayLabel}:${g.id}`)) })
        ] }),
        /* @__PURE__ */ a("div", { className: "workbench-settings-note", children: e.chatSettings.embeddingModel.trim() ? `Embedding alias: ${e.chatSettings.embeddingModel.trim()}` : "Embedding alias: gateway default" }),
        /* @__PURE__ */ a("div", { className: "workbench-settings-note", children: c })
      ] })
    ] }),
    /* @__PURE__ */ u("footer", { className: "workbench-settings-footer", children: [
      /* @__PURE__ */ a("button", { onClick: () => e.onChange(en), children: "Reset" }),
      /* @__PURE__ */ a("button", { onClick: e.onClose, children: "Close tab" })
    ] })
  ] }) });
}
function Er({ outputs: e, clearOutputs: t, deleteArtifact: n, runAiSynthesis: r, captureManualCheck: i, openOutputAsFile: s, previewEditFromOutput: o, canPreviewEdit: c = !1, compact: m = !1 }) {
  const b = Ke();
  return /* @__PURE__ */ u("div", { className: `workbench-panel-list ${m ? "compact" : ""}`, children: [
    /* @__PURE__ */ u("div", { style: { display: "flex", gap: 8, marginBottom: 10 }, children: [
      /* @__PURE__ */ u("button", { className: "workbench-button-subtle", onClick: r, children: [
        /* @__PURE__ */ a(Un, { size: 14 }),
        "AI synthesis"
      ] }),
      /* @__PURE__ */ u("button", { className: "workbench-button-subtle", onClick: i, children: [
        /* @__PURE__ */ a(Hs, { size: 14 }),
        "Capture check"
      ] }),
      /* @__PURE__ */ a("button", { className: "workbench-button-subtle", onClick: t, children: "Clear" })
    ] }),
    e.length === 0 ? /* @__PURE__ */ a("p", { className: "workbench-muted", children: "No outputs yet. Save an AI answer as an artifact or create an AI synthesis." }) : e.map((p) => /* @__PURE__ */ u("div", { className: "workbench-output-card", children: [
      /* @__PURE__ */ u("div", { className: "workbench-output-head", children: [
        /* @__PURE__ */ a("strong", { children: p.title }),
        /* @__PURE__ */ a("span", { children: p.source === "ai" ? `AI · ${p.kind}` : p.kind }),
        /* @__PURE__ */ a("button", { onClick: () => void gt(p.body), children: "Copy" }),
        /* @__PURE__ */ a("button", { onClick: () => s(p), children: "Open as file" }),
        o ? /* @__PURE__ */ a(
          "button",
          {
            className: Da(p.body) ? "workbench-output-edit-cta" : void 0,
            onClick: () => o(p),
            disabled: !c,
            children: Da(p.body) ? "Preview/apply edit" : "Preview edit"
          }
        ) : null,
        p.source === "ai" ? /* @__PURE__ */ a("button", { onClick: () => n(p.id), children: b("chat.delete") }) : null
      ] }),
      /* @__PURE__ */ a(Dr, { body: p.body })
    ] }, p.id))
  ] });
}
function Xl({ edit: e, onApply: t, onOpenAsFile: n, onClose: r }) {
  return /* @__PURE__ */ a("div", { className: "workbench-edit-review-overlay", role: "dialog", "aria-label": "Review AI edit", children: /* @__PURE__ */ u("section", { className: "workbench-edit-review", children: [
    /* @__PURE__ */ u("header", { className: "workbench-edit-review-head", children: [
      /* @__PURE__ */ u("div", { children: [
        /* @__PURE__ */ a("strong", { children: "Review AI edit" }),
        /* @__PURE__ */ a("span", { children: e.fileName })
      ] }),
      /* @__PURE__ */ a("button", { title: "Close", onClick: r, children: /* @__PURE__ */ a(qe, { size: 16 }) })
    ] }),
    /* @__PURE__ */ u("div", { className: "workbench-edit-review-meta", children: [
      /* @__PURE__ */ u("span", { children: [
        "Source: ",
        e.sourceTitle
      ] }),
      /* @__PURE__ */ a("span", { children: e.extractionLabel }),
      /* @__PURE__ */ u("span", { children: [
        "+",
        e.stats.added,
        " / -",
        e.stats.removed,
        " / ~",
        e.stats.changed
      ] })
    ] }),
    /* @__PURE__ */ u("div", { className: "workbench-edit-review-grid", children: [
      /* @__PURE__ */ u("div", { className: "workbench-edit-review-pane", children: [
        /* @__PURE__ */ a("h4", { children: "Current" }),
        /* @__PURE__ */ a("pre", { children: e.originalContent })
      ] }),
      /* @__PURE__ */ u("div", { className: "workbench-edit-review-pane proposed", children: [
        /* @__PURE__ */ a("h4", { children: "Proposed" }),
        /* @__PURE__ */ a("pre", { children: e.proposedContent })
      ] })
    ] }),
    /* @__PURE__ */ u("footer", { className: "workbench-edit-review-actions", children: [
      /* @__PURE__ */ a("button", { className: "workbench-button-subtle", onClick: r, children: "Cancel" }),
      /* @__PURE__ */ a("button", { className: "workbench-button-subtle", onClick: n, children: "Open proposed as file" }),
      /* @__PURE__ */ a("button", { className: "workbench-button-primary", onClick: t, children: "Apply to active file" })
    ] })
  ] }) });
}
function Cl({ patch: e, onApply: t, onOpenAsFiles: n, onClose: r }) {
  const i = e.edits.reduce((s, o) => ({
    added: s.added + o.stats.added,
    removed: s.removed + o.stats.removed,
    changed: s.changed + o.stats.changed
  }), { added: 0, removed: 0, changed: 0 });
  return /* @__PURE__ */ a("div", { className: "workbench-edit-review-overlay", role: "dialog", "aria-label": "Review AI workspace patch", children: /* @__PURE__ */ u("section", { className: "workbench-edit-review workspace", children: [
    /* @__PURE__ */ u("header", { className: "workbench-edit-review-head", children: [
      /* @__PURE__ */ u("div", { children: [
        /* @__PURE__ */ a("strong", { children: "Review AI workspace patch" }),
        /* @__PURE__ */ a("span", { children: e.sourceTitle })
      ] }),
      /* @__PURE__ */ a("button", { title: "Close", onClick: r, children: /* @__PURE__ */ a(qe, { size: 16 }) })
    ] }),
    /* @__PURE__ */ u("div", { className: "workbench-edit-review-meta", children: [
      /* @__PURE__ */ u("span", { children: [
        e.edits.length,
        " files"
      ] }),
      /* @__PURE__ */ u("span", { children: [
        "+",
        i.added,
        " / -",
        i.removed,
        " / ~",
        i.changed
      ] }),
      e.skipped.length > 0 ? /* @__PURE__ */ u("span", { children: [
        e.skipped.length,
        " skipped"
      ] }) : null
    ] }),
    /* @__PURE__ */ u("div", { className: "workbench-workspace-patch-list", children: [
      e.edits.map((s) => /* @__PURE__ */ u("article", { className: "workbench-workspace-patch-card", children: [
        /* @__PURE__ */ u("header", { children: [
          /* @__PURE__ */ a("strong", { children: s.fileName }),
          /* @__PURE__ */ u("span", { children: [
            s.extractionLabel,
            " · +",
            s.stats.added,
            " / -",
            s.stats.removed,
            " / ~",
            s.stats.changed
          ] })
        ] }),
        /* @__PURE__ */ a("pre", { children: Vl(s.proposedContent) })
      ] }, s.fileId)),
      e.skipped.length > 0 ? /* @__PURE__ */ u("article", { className: "workbench-workspace-patch-card skipped", children: [
        /* @__PURE__ */ u("header", { children: [
          /* @__PURE__ */ a("strong", { children: "Skipped" }),
          /* @__PURE__ */ a("span", { children: "Paths not open or hunks did not match" })
        ] }),
        /* @__PURE__ */ a("pre", { children: e.skipped.join(`
`) })
      ] }) : null
    ] }),
    /* @__PURE__ */ u("footer", { className: "workbench-edit-review-actions", children: [
      /* @__PURE__ */ a("button", { className: "workbench-button-subtle", onClick: r, children: "Cancel" }),
      /* @__PURE__ */ a("button", { className: "workbench-button-subtle", onClick: n, children: "Open proposals as files" }),
      /* @__PURE__ */ a("button", { className: "workbench-button-primary", onClick: t, children: "Apply workspace patch" })
    ] })
  ] }) });
}
function Ol(e, t = 9e3) {
  return e.length <= t ? e : `${e.slice(0, t)}

[Atomek clipped ${e.length - t} chars for local agent context]`;
}
function Sl(e, t) {
  const n = [
    "Atomek local agent context:",
    "- You are being launched from Atomek, the TytusOS workspace/chat app.",
    "- Do not write files directly. Return findings, markdown, unified diffs, or fenced replacement blocks. Atomek previews edits before applying."
  ];
  if (e)
    n.push("", `Active file: ${e.path}`, `Language: ${e.language}`, `Dirty: ${e.dirty ? "yes" : "no"}`, "", "Active file content:", "```" + e.language, Ol(e.content), "```");
  else if (t.length > 0) {
    n.push("", `Open editors (${t.length}):`);
    for (const r of t.slice(0, 8)) n.push(`- ${r.path} (${r.language}, ${r.content.length} chars${r.dirty ? ", dirty" : ""})`);
  } else
    n.push("", "No file is open. Ask clarifying questions only if the prompt cannot be answered safely.");
  return n.join(`
`);
}
function Ra(e) {
  if (!e?.path.startsWith("/")) return;
  const t = e.path.lastIndexOf("/");
  return t > 0 ? e.path.slice(0, t) : void 0;
}
function Hl(e) {
  return e.kind === "terminal" ? "Open shell" : `Open ${e.label} in Terminal`;
}
function jl(e, t) {
  return t === e.id ? `${e.label} running…` : "Background review";
}
function qr({
  host: e,
  setStatus: t,
  attachSkillToChat: n,
  saveLocalJobOutput: r,
  activeFile: i,
  openEditors: s,
  variant: o = "sidebar"
}) {
  const [c, m] = T([]), [b, p] = T([]), [v, g] = T(null), [S, M] = T(!1), [E, J] = T(null), [F, ae] = T("Review the active Atomek context. Return concise findings. If you propose edits, output a unified diff or fenced replacement blocks so Atomek can preview before applying."), [me, W] = T(null), [Z, K] = T([]), [V, z] = T(() => sl()), [I, ge] = T([]), [ee, fe] = T([]), [Te, He] = T([]), xe = Se([]), [f, q] = T("task-execute"), [G, k] = T("mission"), O = Z.find((d) => d.status === "running" || d.status === "canceling") ?? Z[0] ?? null, x = o === "dock", L = ue(() => kt(v, lt(F || V?.goal || "", v, V?.teamPresetId)), [F, V?.goal, V?.teamPresetId, v]), R = ue(() => Ot(F || V?.goal || "", v, L.id), [F, V?.goal, L.id, v]), X = R.find((d) => d.id === f) ?? R[1] ?? R[0] ?? null, re = s.filter((d) => d.dirty).length, he = i ? `${i.path} · ${i.language} · ${i.content.length.toLocaleString()} chars${i.dirty ? " · dirty" : ""}` : s.length > 0 ? `${s.length} open editor${s.length === 1 ? "" : "s"}${re ? ` · ${re} dirty` : ""}` : "No active file — jobs run with only your typed prompt.", ve = ue(() => [
    {
      label: "Review active file",
      prompt: "Review the active file for bugs, missing edge cases, confusing UX, and production risks. Return prioritized findings with exact file/line references when possible."
    },
    {
      label: "Plan patch",
      prompt: "Propose the smallest safe patch for the active Atomek context. Output a unified diff only if you are confident; otherwise list the exact files/functions to change."
    },
    {
      label: "Explain architecture",
      prompt: "Explain how the active file/context fits into the TytusOS/Atomek architecture. Point out any integration seams that do not make sense."
    }
  ], []);
  oe(() => {
    xe.current = Te;
  }, [Te]);
  const be = N(async (d = V) => {
    if (!d?.rootPath || !e.missions?.listRuns)
      return He([]), [];
    try {
      const w = await e.missions.listRuns(d.rootPath);
      return He(w), xe.current = w, w;
    } catch (w) {
      return t(`Mission run history failed: ${w instanceof Error ? w.message : String(w)}`), [];
    }
  }, [e.missions, V, t]), Ue = N(async (d, w) => {
    if (!d) return;
    const A = [...w].sort((H, D) => Ia(D).localeCompare(Ia(H))).slice(0, 100), C = A.map((H) => JSON.stringify(H)).join(`
`) + (A.length ? `
` : "");
    d.rootPath && e.missions?.write ? await e.missions.write({ rootPath: d.rootPath, files: [{ path: "RUNS.jsonl", content: C }] }) : d.handle && await Jn(d.handle, "RUNS.jsonl", C), He(A), xe.current = A;
  }, [e.missions]), ke = N(async (d, w = V) => {
    if (!w) return;
    const A = [
      d,
      ...xe.current.filter((C) => C.id !== d.id)
    ];
    await Ue(w, A);
  }, [V, Ue]), Be = N(async () => {
    if (!e.local?.listTools && !e.skills?.list && !e.resources?.list) {
      m([]), p([]), g(null), J("This Tytus host build does not expose local tools, resource graph, or skill registry yet.");
      return;
    }
    M(!0), J(null);
    try {
      const [d, w, A, C] = await Promise.all([
        e.local?.listTools?.().catch((H) => (t(`Local tool discovery failed: ${H instanceof Error ? H.message : String(H)}`), [])) ?? Promise.resolve([]),
        e.skills?.list?.().catch((H) => (t(`Skill registry discovery failed: ${H instanceof Error ? H.message : String(H)}`), [])) ?? Promise.resolve([]),
        e.resources?.list?.().catch((H) => (t(`Resource graph discovery failed: ${H instanceof Error ? H.message : String(H)}`), null)) ?? Promise.resolve(null),
        e.missions?.list?.().catch((H) => (t(`Mission list failed: ${H instanceof Error ? H.message : String(H)}`), [])) ?? Promise.resolve([])
      ]);
      m(d), p(w), g(A), ge(C), V?.rootPath && be(V), t(`Agent team loaded · ${d.length} tools · ${w.length} skills · ${C.length} missions${A ? ` · ${A.resources.length} resources` : ""}`);
    } catch (d) {
      J(d instanceof Error ? d.message : String(d));
    } finally {
      M(!1);
    }
  }, [e.local, e.missions, e.resources, e.skills, be, V, t]);
  oe(() => {
    Be();
  }, [Be]), oe(() => {
    const d = (w) => {
      const A = w.detail;
      A?.missionId && (z(A), be(A)), Be();
    };
    return window.addEventListener(Yn, d), () => window.removeEventListener(Yn, d);
  }, [Be, be]);
  const at = N((d) => {
    const w = il(d);
    z(w), Lt(w), be(w), fe([{
      ts: (/* @__PURE__ */ new Date()).toISOString(),
      kind: "mission.resume",
      message: `Mission resumed from Atomek agent team: ${w.rootPath ?? w.name}`,
      data: { runCount: d.runCount ?? 0, taskCount: d.taskCount ?? 0 }
    }]), ae(d.goal || `Continue mission ${d.title}. Review MISSION.md, TASKS.md, RESOURCES.md, and runs/ before acting.`), t(`Resumed mission: ${d.rootPath}`);
  }, [be, t]), Pe = N(async (d, w, A = []) => {
    const C = [
      ...ee,
      ...A,
      { ts: (/* @__PURE__ */ new Date()).toISOString(), kind: "mission.pack.write", message: "Mission context pack written from Atomek" }
    ], H = lt(w || d.goal, v, d.teamPresetId), D = Ot(w || d.goal, v, H), B = [
      { path: "MISSION.md", content: tn(d, v, i, s, w, H) },
      { path: "MISSION.json", content: Vr(d, v, w, H) },
      { path: "RESOURCES.md", content: nn(v) },
      { path: "TASKS.md", content: Mr(D) },
      { path: "HANDOFF.md", content: Wr(d) },
      { path: "INBOX.md", content: `# Mission inbox

Drop incoming agent notes, pod outputs, and shared-folder discoveries here.
` },
      { path: "OUTBOX.md", content: `# Mission outbox

Approved handoffs, final artifacts, and user-ready summaries go here.
` },
      { path: "AUDIT.jsonl", content: C.map((ie) => JSON.stringify(ie)).join(`
`) + `
` },
      { path: "RUNS.jsonl", content: xe.current.map((ie) => JSON.stringify(ie)).join(`
`) + (xe.current.length ? `
` : "") },
      { path: "runs/README.md", content: `# Mission runs

Local, pod, and app run transcripts land here.
` },
      { path: "outputs/README.md", content: `# Mission outputs

Final artifacts and generated files land here before handoff.
` },
      { path: "proposals/README.md", content: `# Mission proposals

Patch/write/publish proposals land here before approval.
` },
      { path: "approvals/README.md", content: `# Mission approvals

Approval and rejection decisions reference proposal files from here.
` }
    ];
    if (d.rootPath && e.missions?.write)
      await e.missions.write({ rootPath: d.rootPath, files: B });
    else if (d.handle) {
      await pt(d.handle, "runs"), await pt(d.handle, "outputs"), await pt(d.handle, "proposals"), await pt(d.handle, "approvals");
      for (const ie of B) await $c(d.handle, ie.path, ie.content);
    } else
      throw new Error("Mission has neither tray rootPath nor browser folder handle");
    fe(C), Lt(d);
  }, [i, e.missions, ee, s, v]), ze = N(async (d, w = {}) => {
    if (V)
      return await Pe(V, d), V;
    const A = `Atomek mission ${(/* @__PURE__ */ new Date()).toLocaleString()}`, C = d.trim() || "Coordinate Tytus resources for the current Atomek task.";
    let H = null;
    if (e.missions?.create) {
      const B = await e.missions.create({ title: A, goal: C });
      H = {
        missionId: B.missionId,
        title: B.title,
        goal: B.goal,
        rootPath: B.rootPath,
        name: B.rootPath.split("/").pop() || B.missionId,
        source: "tray"
      };
    } else if (w.allowBrowserPicker) {
      const B = await Qc();
      if (!B)
        return t("Mission folder picker unavailable in this browser context"), null;
      H = {
        handle: B,
        name: B.name,
        missionId: `mission-${Date.now()}-${rl(B.name)}`,
        title: A,
        goal: C,
        source: "browser"
      };
    }
    if (!H) return null;
    const D = { ts: (/* @__PURE__ */ new Date()).toISOString(), kind: "mission.folder.ready", message: `Mission folder ready: ${H.rootPath ?? H.name}` };
    return z(H), Lt(H), fe([D]), await Pe(H, C, [D]), t(`Mission pack ready in ${H.rootPath ?? H.name}`), H;
  }, [e.missions, V, t, Pe]), Ye = N(async (d) => {
    if (!e.local?.openTerminal) {
      t("Terminal bridge unavailable in this host build");
      return;
    }
    try {
      const w = F.trim() || `Open ${d.label} from Atomek with current context.`, A = d.kind === "ai-cli" ? await ze(w) : V;
      await e.local.openTerminal({
        toolId: d.id,
        command: d.command,
        cwd: A?.rootPath ?? Ra(i),
        prompt: A ? `Atomek mission pack ready at ${A.rootPath ?? A.name}. Read MISSION.md and RESOURCES.md. ${he}` : `Opened from Atomek. ${he}`
      }), t(d.kind === "ai-cli" ? `Started ${d.label} in a fresh Tytus Terminal with mission context.` : "Opened Tytus Terminal");
    } catch (w) {
      t(`Terminal launch failed: ${w instanceof Error ? w.message : String(w)}`);
    }
  }, [i, he, ze, e.local, F, V, t]), te = N(async () => {
    try {
      await ze(F.trim(), { allowBrowserPicker: !0 }) || t("Mission folder setup skipped.");
    } catch (d) {
      t(`Mission folder setup failed: ${d instanceof Error ? d.message : String(d)}`);
    }
  }, [ze, F, t]), U = N(async (d, w, A, C = V, H) => {
    if (!C) return;
    const D = (/* @__PURE__ */ new Date()).toISOString().replace(/[:.]/g, "-"), B = H ?? `runs/${D}-${d.id}.md`, ie = B.split("/").pop() || `${D}-${d.id}.md`;
    if (C.rootPath && e.missions?.write)
      await e.missions.write({ rootPath: C.rootPath, files: [{ path: B, content: w }] });
    else if (C.handle) {
      const Xe = await pt(C.handle, "runs");
      await Jn(Xe, ie, w);
    }
    await Pe(C, F, [{
      ts: (/* @__PURE__ */ new Date()).toISOString(),
      kind: "local-cli.run.complete",
      message: `${d.label} exited ${A}; transcript saved to ${B}`,
      data: { toolId: d.id, exitCode: A, transcript: B }
    }]);
  }, [e.missions, F, V, Pe]), ce = N(async (d) => {
    if (!e.local?.runJob || !e.local?.streamJob) {
      t("Local job runner unavailable in this host build");
      return;
    }
    const w = F.trim();
    if (!w) {
      t("Local job prompt is empty");
      return;
    }
    let A = null;
    try {
      A = await ze(w), A && await Pe(A, w, [{
        ts: (/* @__PURE__ */ new Date()).toISOString(),
        kind: "local-cli.run.start",
        message: `${d.label} background review started`,
        data: { toolId: d.id, taskId: X?.id ?? "manual", taskTitle: X?.title ?? "Manual run" }
      }]);
    } catch (D) {
      t(`Mission pack failed before local job start: ${D instanceof Error ? D.message : String(D)}`);
      return;
    }
    W(d.id);
    const C = `local-run-${Date.now()}-${d.id}`, H = ft();
    K((D) => [{
      id: C,
      toolId: d.id,
      label: d.label,
      status: "running",
      startedAt: Date.now(),
      taskId: X?.id ?? "manual",
      taskTitle: X?.title ?? "Manual run",
      lines: [`[Atomek] Starting ${d.label} local job for ${X?.title ?? "manual run"}…`]
    }, ...D].slice(0, 6));
    try {
      const D = await e.local.runJob({
        toolId: d.id,
        prompt: A ? [
          "Tytus mission context pack is active.",
          `Mission: ${A.title}`,
          `Goal: ${A.goal}`,
          A.rootPath ? `Mission folder: ${A.rootPath}` : `Mission folder: ${A.name}`,
          "Read MISSION.md and RESOURCES.md from the mission folder when available.",
          "Use the attached Atomek context as source of truth. If you propose file writes, return a unified diff/replacement only; Atomek approval gate applies it.",
          "",
          w
        ].join(`
`) : w,
        cwd: A?.rootPath ?? Ra(i),
        mission: A ? {
          missionId: A.missionId,
          rootPath: A.rootPath,
          taskId: X?.id ?? "manual",
          taskTitle: X?.title ?? "Manual run",
          resourceId: d.id
        } : void 0,
        context: [
          A ? tn(A, v, i, s, w) : "",
          Sl(i, s),
          v ? nn(v) : ""
        ].filter(Boolean).join(`

---

`)
      }), B = D.transcriptPath ?? (A?.rootPath ? `runs/${(/* @__PURE__ */ new Date()).toISOString().replace(/[:.]/g, "-")}-${d.id}.md` : void 0), ie = [
        `[Atomek] Started ${d.label} local job ${D.id}`,
        X ? `[Atomek] Task: ${X.title} (${X.id})` : "[Atomek] Task: manual",
        A?.rootPath ? `[Atomek] Mission: ${A.rootPath}` : ""
      ].filter(Boolean), Xe = {
        id: C,
        jobId: D.id,
        toolId: d.id,
        label: d.label,
        status: "running",
        startedAt: H,
        taskId: X?.id ?? "manual",
        taskTitle: X?.title ?? "Manual run",
        transcriptPath: B,
        summary: `Started ${d.label} for ${X?.title ?? "manual run"}`
      };
      ke(Xe, A).catch((le) => {
        t(`Mission run index failed: ${le instanceof Error ? le.message : String(le)}`);
      }), K((le) => le.map(($) => $.id === C ? { ...$, jobId: D.id, transcriptPath: B } : $));
      const Le = (le) => {
        K(($) => $.map((_) => _.id === C ? le(_) : _));
      };
      e.local.streamJob(D.id, {
        onLog: (le) => {
          ie.push(le), Le(($) => ({ ...$, lines: ie.slice(-500) }));
        },
        onDone: (le) => {
          le && (ie.push(le), Le(($) => ({ ...$, lines: ie.slice(-500) })));
        },
        onFail: (le) => {
          ie.push(`[FAIL] ${le}`), Le(($) => ({
            ...$,
            status: "failed",
            finishedAt: Date.now(),
            lines: ie.slice(-500)
          })), ke({
            ...Xe,
            status: "failed",
            finishedAt: ft(),
            exitCode: -1,
            summary: le
          }, A).catch(($) => {
            t(`Mission run index failed: ${$ instanceof Error ? $.message : String($)}`);
          }), r(`${d.label} local job failed`, ie.join(`
`)), U(d, ie.join(`
`), -1, A, B).catch(($) => {
            t(`Mission transcript save failed: ${$ instanceof Error ? $.message : String($)}`);
          }), W(null);
        },
        onExit: (le) => {
          const $ = [
            `# Local job — ${d.label}`,
            "",
            `- Tool: ${d.id}`,
            `- Exit code: ${le}`,
            `- Captured: ${(/* @__PURE__ */ new Date()).toISOString()}`,
            "",
            "```text",
            ie.join(`
`),
            "```"
          ].join(`
`);
          Le((_) => ({
            ..._,
            status: le === 0 ? "complete" : "failed",
            exitCode: le,
            finishedAt: Date.now(),
            lines: ie.slice(-500)
          })), ke({
            ...Xe,
            status: le === 0 ? "complete" : "failed",
            finishedAt: ft(),
            exitCode: le,
            summary: `${d.label} exited ${le}`
          }, A).catch((_) => {
            t(`Mission run index failed: ${_ instanceof Error ? _.message : String(_)}`);
          }), r(`${d.label} local job`, $), U(d, $, le, A, B).catch((_) => {
            t(`Mission transcript save failed: ${_ instanceof Error ? _.message : String(_)}`);
          }), W(null);
        },
        onError: () => t(`Local job stream issue for ${d.label}`)
      }), t(`Started ${d.label} mission run${X ? ` · ${X.title}` : ""}`);
    } catch (D) {
      W(null), K((B) => B.map((ie) => ie.id === C ? {
        ...ie,
        status: "failed",
        finishedAt: Date.now(),
        lines: [...ie.lines, `[Atomek] Failed to start: ${D instanceof Error ? D.message : String(D)}`]
      } : ie)), t(`Local job failed to start: ${D instanceof Error ? D.message : String(D)}`);
    }
  }, [i, ze, e.local, F, s, v, r, U, X, t, ke, Pe]), se = N(async (d) => {
    const w = Zn(d), A = _c(d);
    if (!w) {
      t(`Cannot dispatch ${je(d)}: missing pod id`);
      return;
    }
    const C = F.trim();
    if (!C) {
      t("Pod task prompt is empty");
      return;
    }
    let H = null;
    const D = je(d), B = `pod-run-${Date.now()}-${w}`, ie = ft(), Xe = `runs/${(/* @__PURE__ */ new Date()).toISOString().replace(/[:.]/g, "-")}-${d.id.replace(/[^a-z0-9_.-]/gi, "-")}.md`;
    try {
      H = await ze(C), H && await Pe(H, C, [{
        ts: (/* @__PURE__ */ new Date()).toISOString(),
        kind: "pod-agent.run.start",
        message: `${D} mission task started`,
        data: { resourceId: d.id, podId: w, routeId: A ?? void 0, taskId: X?.id ?? "manual", taskTitle: X?.title ?? "Manual pod run" }
      }]);
    } catch (_) {
      t(`Mission pack failed before pod task: ${_ instanceof Error ? _.message : String(_)}`);
      return;
    }
    const Le = {
      id: B,
      toolId: d.id,
      label: D,
      status: "running",
      startedAt: ie,
      taskId: X?.id ?? "manual",
      taskTitle: X?.title ?? "Manual pod run",
      transcriptPath: Xe,
      summary: `Started ${D} for ${X?.title ?? "manual pod run"}`
    };
    K((_) => [{
      id: B,
      toolId: d.id,
      label: D,
      status: "running",
      startedAt: Date.now(),
      taskId: X?.id ?? "manual",
      taskTitle: X?.title ?? "Manual pod run",
      transcriptPath: Xe,
      lines: [`[Atomek] Dispatching ${D} via Tytus host bridge…`, `[Atomek] Pod: ${w}`, `[Atomek] Task: ${X?.title ?? "manual pod run"}`]
    }, ..._].slice(0, 6)), await ke(Le, H).catch((_) => {
      t(`Mission run index failed: ${_ instanceof Error ? _.message : String(_)}`);
    });
    const le = [], $ = (_) => {
      le.push(_), K((Ne) => Ne.map((We) => We.id === B ? { ...We, lines: [...We.lines, _].slice(-500) } : We));
    };
    try {
      $("[Atomek] Routing through pod-agent chat bridge.");
      const _ = [
        "You are a Tytus pod agent working from an Atomek mission pack.",
        "Use only the mission/shared-folder context described by the user.",
        "Return findings, markdown, or patch proposals. Do not claim direct writes.",
        "If you propose edits, output unified diff or fenced replacement blocks for Atomek approval.",
        "",
        H ? `Mission folder: ${H.rootPath ?? H.name}` : "Mission folder: not available",
        H ? `Mission: ${H.title}` : "",
        H ? `Goal: ${H.goal}` : "",
        X ? `Task: ${X.title} (${X.id})` : "Task: manual",
        "",
        "Mission context:",
        H ? tn(H, v, i, s, C) : "",
        v ? nn(v) : "",
        "",
        "User task:",
        C
      ].filter(Boolean).join(`
`);
      let Ne = "", We = !1;
      for await (const ye of e.daemon.chatAgent({
        podId: w,
        routeId: A,
        message: _,
        mode: "operator",
        target: "agent",
        modelPreference: "balanced"
      })) {
        if (ye.type === "profile" && $(`[Atomek] ${ye.profile === "local" ? "Local Cortex" : "Cloud Cortex"} route selected.`), ye.type === "session" && $("[Atomek] Pod-agent session established."), ye.type === "token" && (We || ($("[Atomek] Pod agent is responding…"), We = !0), Ne = ln(`${Ne}${ye.text}`)), ye.type === "error") {
          const Ve = Dn(ye.message);
          throw new Error(Ve.message);
        }
        if (ye.type === "done") break;
      }
      if (Ne = Ne.trim(), !Ne) throw new Error("Pod agent returned no text.");
      $(Ne);
      const Ce = [
        `# Pod job — ${D}`,
        "",
        `- Resource: ${d.id}`,
        `- Pod: ${w}`,
        `- Captured: ${(/* @__PURE__ */ new Date()).toISOString()}`,
        `- Task: ${X?.title ?? "Manual pod run"}`,
        "",
        Ne,
        ""
      ].join(`
`);
      K((ye) => ye.map((Ve) => Ve.id === B ? { ...Ve, status: "complete", finishedAt: Date.now(), lines: [...Ve.lines, "[Atomek] Pod task complete."].slice(-500) } : Ve)), await ke({ ...Le, status: "complete", finishedAt: ft(), summary: `${D} completed`, transcriptPath: Xe }, H), r(`${D} pod job`, Ce), await U({ id: d.id, label: D, kind: "pod-agent", status: d.status }, Ce, 0, H, Xe), t(`${D} completed mission task`);
    } catch (_) {
      const Ne = _ instanceof Error ? _.message : String(_);
      $(`[FAIL] ${Ne}`), K((Ce) => Ce.map((ye) => ye.id === B ? { ...ye, status: "failed", finishedAt: Date.now(), lines: ye.lines.slice(-500) } : ye)), await ke({ ...Le, status: "failed", finishedAt: ft(), exitCode: -1, summary: Ne, transcriptPath: Xe }, H).catch((Ce) => {
        t(`Mission run index failed: ${Ce instanceof Error ? Ce.message : String(Ce)}`);
      });
      const We = [`# Pod job failed — ${D}`, "", `- Pod: ${w}`, `- Error: ${Ne}`, "", "```text", le.join(`
`), "```", ""].join(`
`);
      r(`${D} pod job failed`, We), await U({ id: d.id, label: D, kind: "pod-agent", status: d.status }, We, -1, H, Xe).catch((Ce) => {
        t(`Pod transcript save failed: ${Ce instanceof Error ? Ce.message : String(Ce)}`);
      }), t(`Pod task failed: ${Ne}`);
    }
  }, [i, ze, e.daemon, F, s, v, r, U, X, t, ke, Pe]), Ae = N(async (d) => {
    if (!d.jobId || !e.local?.cancelJob) {
      t("Local job cancel bridge unavailable in this host build");
      return;
    }
    K((w) => w.map((A) => A.id === d.id ? { ...A, status: "canceling", lines: [...A.lines, "[Atomek] Cancel requested…"] } : A)), ke({
      id: d.id,
      jobId: d.jobId,
      toolId: d.toolId,
      label: d.label,
      status: "canceling",
      startedAt: new Date(d.startedAt).toISOString(),
      taskId: d.taskId,
      taskTitle: d.taskTitle,
      transcriptPath: d.transcriptPath,
      summary: "Cancel requested from Atomek"
    }).catch((w) => {
      t(`Mission run index failed: ${w instanceof Error ? w.message : String(w)}`);
    });
    try {
      await e.local.cancelJob(d.jobId), t(`Cancel requested for ${d.label}`);
    } catch (w) {
      K((A) => A.map((C) => C.id === d.id ? { ...C, status: "running", lines: [...C.lines, `[Atomek] Cancel failed: ${w instanceof Error ? w.message : String(w)}`] } : C)), t(`Local job cancel failed: ${w instanceof Error ? w.message : String(w)}`);
    }
  }, [e.local, t, ke]), Me = N(async (d) => {
    const w = [
      `Use Tytus resource "${je(d)}" (${d.kind}) for the next mission step.`,
      `Capabilities: ${d.capabilities.join(", ") || "status only"}.`,
      `Sandbox: ${d.sandbox}. Trust: ${d.trustTier}.`,
      d.allowedRoots.length ? `Allowed roots: ${d.allowedRoots.join(", ")}` : "No direct roots exposed.",
      "Return transcript/findings/artifacts only; edits require Atomek approval."
    ].join(`
`);
    ae(w), V && await Pe(V, w, [{
      ts: (/* @__PURE__ */ new Date()).toISOString(),
      kind: "resource.selected",
      message: `Selected resource ${je(d)}`,
      data: { resourceId: d.id, kind: d.kind, status: d.status }
    }]), t(`Selected ${je(d)} for mission`);
  }, [V, t, Pe]), Ge = N((d) => {
    const w = d.setupAction, A = w?.commandPreview ? `${w.label}: ${w.commandPreview}` : w?.deepLink ? `${w.label}: ${w.deepLink}` : w?.label ?? `${je(d)} needs setup`;
    t(A), gt(w?.commandPreview ?? w?.deepLink ?? A);
  }, [t]);
  return /* @__PURE__ */ u("aside", { className: x ? "workbench-agent-dock" : "workbench-sidebar", children: [
    x ? null : /* @__PURE__ */ a("div", { className: "workbench-sidebar-title", children: "AGENT TEAM" }),
    /* @__PURE__ */ u("div", { className: x ? "workbench-agent-dock-scroll" : "workbench-sidebar-scroll", children: [
      /* @__PURE__ */ u("div", { className: "workbench-computer-hero", children: [
        /* @__PURE__ */ a(Un, { size: 18 }),
        /* @__PURE__ */ u("div", { children: [
          /* @__PURE__ */ a("strong", { children: x ? "Mission Runs" : "OpenClaw + Hermes Team Board" }),
          /* @__PURE__ */ a("p", { className: "workbench-muted", children: "Coordinate OpenClaw pods, Hermes pods, local agents, shared folders, app skills, and AIL routes. Open tools in Terminal when you want hands-on control; run background reviews when you want streamed, approval-gated output." })
        ] })
      ] }),
      /* @__PURE__ */ u("button", { className: "workbench-button-subtle workbench-computer-refresh", onClick: () => {
        Be();
      }, disabled: S, children: [
        /* @__PURE__ */ a(bt, { size: 14 }),
        " ",
        S ? "Refreshing…" : "Refresh capabilities"
      ] }),
      E && /* @__PURE__ */ a("div", { className: "workbench-inline-error", children: E }),
      /* @__PURE__ */ u("div", { className: "workbench-team-switcher", role: "tablist", "aria-label": "Atomek agent team views", children: [
        /* @__PURE__ */ a("button", { className: G === "mission" ? "active" : "", onClick: () => k("mission"), children: "Mission" }),
        /* @__PURE__ */ a("button", { className: G === "runs" ? "active" : "", onClick: () => k("runs"), children: "Runs" }),
        /* @__PURE__ */ a("button", { className: G === "setup" ? "active" : "", onClick: () => k("setup"), children: "Setup" })
      ] }),
      G === "mission" ? /* @__PURE__ */ u(De, { children: [
        /* @__PURE__ */ a("div", { className: "workbench-section-title", children: "MISSION PACK — SHARED CONTEXT" }),
        /* @__PURE__ */ u("div", { className: "workbench-computer-context-card mission", children: [
          /* @__PURE__ */ a("strong", { children: V ? V.title : "No mission folder selected" }),
          /* @__PURE__ */ a("span", { children: V ? `${V.rootPath ?? V.name} · ${V.source} · ${ee.length} audit events · transcripts saved under runs/` : "Atomek creates this automatically before launching local agents. It is the shared folder agents read/write transcripts from." }),
          v ? /* @__PURE__ */ u("span", { children: [
            Hr(v.resources),
            v.warnings.length ? ` · ${v.warnings.length} warnings` : ""
          ] }) : /* @__PURE__ */ a("span", { children: "Resource graph not loaded yet." })
        ] }),
        /* @__PURE__ */ a("div", { className: "workbench-section-title", children: "SELECTED TEAM" }),
        /* @__PURE__ */ u("div", { className: "workbench-team-assignment-list", children: [
          /* @__PURE__ */ u("div", { className: `workbench-team-assignment-summary ${L.readiness}`, children: [
            /* @__PURE__ */ a("strong", { children: L.label }),
            /* @__PURE__ */ u("span", { children: [
              L.readiness,
              " · ",
              L.bestFor
            ] })
          ] }),
          L.assignments.map((d) => /* @__PURE__ */ u("div", { className: "workbench-team-assignment-row", children: [
            /* @__PURE__ */ u("div", { children: [
              /* @__PURE__ */ a("strong", { children: d.label }),
              /* @__PURE__ */ a("span", { children: d.resourceLabel })
            ] }),
            /* @__PURE__ */ a("em", { children: d.status })
          ] }, `${d.role}-${d.resourceId}`))
        ] }),
        /* @__PURE__ */ u("div", { className: "workbench-computer-actions", children: [
          /* @__PURE__ */ a("button", { className: "workbench-button-subtle workbench-agent-primary-action", onClick: () => {
            te();
          }, children: V ? "Refresh mission pack" : "Start mission pack" }),
          /* @__PURE__ */ a("button", { className: "workbench-button-subtle", onClick: () => V && void Pe(V, F), disabled: !V, children: "Rewrite context files" })
        ] }),
        I.length ? /* @__PURE__ */ u(De, { children: [
          /* @__PURE__ */ a("div", { className: "workbench-section-title", children: "RESUME MISSION" }),
          /* @__PURE__ */ a("div", { className: "workbench-mission-list", children: I.slice(0, x ? 3 : 5).map((d) => /* @__PURE__ */ u(
            "button",
            {
              className: `workbench-mission-row ${V?.missionId === d.missionId ? "active" : ""}`,
              onClick: () => at(d),
              title: d.rootPath,
              children: [
                /* @__PURE__ */ a("strong", { children: d.title }),
                /* @__PURE__ */ u("span", { children: [
                  d.status ?? "active",
                  " · ",
                  d.taskCount ?? 0,
                  " tasks · ",
                  d.runCount ?? 0,
                  " runs"
                ] })
              ]
            },
            d.missionId
          )) })
        ] }) : null,
        v?.warnings.length ? /* @__PURE__ */ a("div", { className: "workbench-resource-warnings", children: v.warnings.slice(0, 3).map((d) => /* @__PURE__ */ u("span", { children: [
          d.code,
          ": ",
          d.message
        ] }, `${d.code}-${d.resourceId ?? d.message}`)) }) : null,
        /* @__PURE__ */ a("div", { className: "workbench-section-title", children: "ACTIVE CONTEXT" }),
        /* @__PURE__ */ u("div", { className: "workbench-computer-context-card", children: [
          /* @__PURE__ */ a("strong", { children: he }),
          /* @__PURE__ */ a("span", { children: i ? "Local agents receive clipped active-file content and must return previewable edits." : "Open a file to give local agents useful context." })
        ] }),
        /* @__PURE__ */ a("div", { className: "workbench-section-title", children: "TASK FOR LOCAL AGENT" }),
        /* @__PURE__ */ a("div", { className: "workbench-computer-presets", children: ve.map((d) => /* @__PURE__ */ a("button", { className: "workbench-button-subtle", onClick: () => ae(d.prompt), children: d.label }, d.label)) }),
        /* @__PURE__ */ a(
          "textarea",
          {
            className: "workbench-computer-job-prompt",
            value: F,
            onChange: (d) => ae(d.target.value),
            rows: 5
          }
        ),
        /* @__PURE__ */ a("div", { className: "workbench-section-title", children: "TASK GRAPH" }),
        /* @__PURE__ */ a("div", { className: "workbench-task-graph", children: R.map((d, w) => /* @__PURE__ */ u(
          "button",
          {
            className: `workbench-task-card ${d.status} ${X?.id === d.id ? "active" : ""}`,
            onClick: () => {
              q(d.id), ae(d.prompt);
            },
            title: "Load this task prompt",
            children: [
              /* @__PURE__ */ a("span", { children: w + 1 }),
              /* @__PURE__ */ a("strong", { children: d.title }),
              /* @__PURE__ */ a("em", { children: d.resourceHint }),
              /* @__PURE__ */ a("small", { children: d.assignedResourceLabel })
            ]
          },
          d.id
        )) })
      ] }) : null,
      G === "runs" ? /* @__PURE__ */ u(De, { children: [
        O ? /* @__PURE__ */ u("div", { className: "workbench-agent-run", children: [
          /* @__PURE__ */ u("header", { children: [
            /* @__PURE__ */ u("div", { children: [
              /* @__PURE__ */ a("strong", { children: O.label }),
              /* @__PURE__ */ u("span", { children: [
                O.status,
                typeof O.exitCode == "number" ? ` · exit ${O.exitCode}` : "",
                O.taskTitle ? ` · ${O.taskTitle}` : ""
              ] }),
              O.transcriptPath ? /* @__PURE__ */ a("span", { children: O.transcriptPath }) : null
            ] }),
            /* @__PURE__ */ u("div", { className: "workbench-agent-run-actions", children: [
              /* @__PURE__ */ a("button", { className: "workbench-button-subtle", onClick: () => r(`${O.label} local job`, O.lines.join(`
`)), disabled: O.lines.length === 0, children: "Save output" }),
              O.jobId && (O.status === "running" || O.status === "canceling") ? /* @__PURE__ */ u("button", { className: "workbench-button-subtle danger", onClick: () => {
                Ae(O);
              }, disabled: O.status === "canceling", children: [
                /* @__PURE__ */ a(Ga, { size: 12 }),
                " ",
                O.status === "canceling" ? "Canceling…" : "Cancel"
              ] }) : null
            ] })
          ] }),
          /* @__PURE__ */ a("pre", { className: "workbench-computer-job-log", children: O.lines.join(`
`) || "[waiting for output]" })
        ] }) : null,
        Te.length ? /* @__PURE__ */ u(De, { children: [
          /* @__PURE__ */ a("div", { className: "workbench-section-title", children: "RUN HISTORY" }),
          /* @__PURE__ */ a("div", { className: "workbench-run-history", children: Te.slice(0, x ? 4 : 8).map((d) => /* @__PURE__ */ u("div", { className: "workbench-run-history-row", children: [
            /* @__PURE__ */ u("div", { children: [
              /* @__PURE__ */ a("strong", { children: d.taskTitle || d.label }),
              /* @__PURE__ */ u("span", { children: [
                d.label,
                " · ",
                d.status,
                typeof d.exitCode == "number" ? ` · exit ${d.exitCode}` : ""
              ] }),
              d.transcriptPath ? /* @__PURE__ */ a("small", { children: d.transcriptPath }) : null
            ] }),
            /* @__PURE__ */ a("button", { className: "workbench-button-subtle", onClick: () => void gt(d.transcriptPath ?? d.id), title: "Copy transcript path or run id", children: "Copy path" })
          ] }, d.id)) })
        ] }) : null
      ] }) : null,
      G === "setup" ? /* @__PURE__ */ u(De, { children: [
        /* @__PURE__ */ a("div", { className: "workbench-section-title", children: "RESOURCE GRAPH" }),
        /* @__PURE__ */ u("div", { className: "workbench-computer-list compact", children: [
          !v && !S ? /* @__PURE__ */ a("p", { className: "workbench-muted", children: "No resource graph reported yet. Older Tytus host builds need `/api/resources`." }) : null,
          v?.resources.slice(0, x ? 8 : 5).map((d) => /* @__PURE__ */ u("div", { className: "workbench-resource-row", children: [
            /* @__PURE__ */ u("div", { children: [
              /* @__PURE__ */ a("strong", { children: je(d) }),
              /* @__PURE__ */ a("span", { children: oa(d) })
            ] }),
            /* @__PURE__ */ u("div", { className: "workbench-resource-row-actions", children: [
              /* @__PURE__ */ a("span", { className: `workbench-computer-pill ${d.status}`, children: d.status }),
              /* @__PURE__ */ a("button", { className: "workbench-button-subtle", onClick: () => {
                Me(d);
              }, children: "Use" }),
              d.kind === "pod-agent" && Zn(d) ? /* @__PURE__ */ a("button", { className: "workbench-button-subtle", onClick: () => {
                se(d);
              }, children: "Ask pod" }) : null,
              d.status === "needs-setup" || d.setupAction ? /* @__PURE__ */ a("button", { className: "workbench-button-subtle", onClick: () => Ge(d), children: "Setup" }) : null
            ] })
          ] }, d.id))
        ] }),
        /* @__PURE__ */ u("div", { className: "workbench-computer-explainer", children: [
          /* @__PURE__ */ a("strong", { children: "How launch works" }),
          /* @__PURE__ */ u("span", { children: [
            /* @__PURE__ */ a("b", { children: "Open in Terminal" }),
            ": opens a fresh Tytus Terminal, changes into the mission folder, and starts the selected CLI immediately."
          ] }),
          /* @__PURE__ */ u("span", { children: [
            /* @__PURE__ */ a("b", { children: "Background review" }),
            ": runs the CLI through the tray in read-only/planning mode, streams output here, saves a transcript under ",
            /* @__PURE__ */ a("code", { children: "runs/" }),
            ", and never applies edits directly."
          ] })
        ] }),
        /* @__PURE__ */ a("div", { className: "workbench-section-title", children: "LOCAL AGENTS & TERMINAL" }),
        /* @__PURE__ */ u("div", { className: "workbench-computer-list", children: [
          c.length === 0 && !S ? /* @__PURE__ */ a("p", { className: "workbench-muted", children: "No local tools reported yet." }) : null,
          c.map((d) => /* @__PURE__ */ u("div", { className: "workbench-computer-card", children: [
            /* @__PURE__ */ u("div", { className: "workbench-computer-card-head", children: [
              /* @__PURE__ */ u("div", { children: [
                /* @__PURE__ */ a("strong", { children: d.label }),
                /* @__PURE__ */ u("span", { children: [
                  d.kind,
                  d.version ? ` · ${d.version}` : ""
                ] })
              ] }),
              /* @__PURE__ */ a("span", { className: `workbench-computer-pill ${d.status}`, children: d.status })
            ] }),
            d.description ? /* @__PURE__ */ a("p", { className: "workbench-muted", children: d.description }) : null,
            /* @__PURE__ */ u("div", { className: "workbench-computer-actions", children: [
              /* @__PURE__ */ a("button", { className: "workbench-button-subtle workbench-agent-primary-action", onClick: () => {
                Ye(d);
              }, disabled: d.status !== "available", title: "Launch this tool in a fresh Tytus Terminal with mission context and execute it immediately.", children: Hl(d) }),
              d.kind === "ai-cli" ? /* @__PURE__ */ a("button", { className: "workbench-button-subtle", onClick: () => {
                ce(d);
              }, disabled: d.status !== "available" || me !== null, title: "Run this local agent as a background read-only review inside Atomek and stream output here. It cannot write files directly.", children: jl(d, me) }) : null
            ] })
          ] }, d.id))
        ] }),
        /* @__PURE__ */ a("div", { className: "workbench-section-title", children: "AGENTIC APP SKILLS" }),
        /* @__PURE__ */ u("div", { className: "workbench-computer-list", children: [
          b.length === 0 && !S ? /* @__PURE__ */ a("p", { className: "workbench-muted", children: "No skills reported yet." }) : null,
          b.map((d) => /* @__PURE__ */ u("div", { className: "workbench-computer-card", children: [
            /* @__PURE__ */ u("div", { className: "workbench-computer-card-head", children: [
              /* @__PURE__ */ u("div", { children: [
                /* @__PURE__ */ a("strong", { children: d.title }),
                /* @__PURE__ */ u("span", { children: [
                  d.driver,
                  " · ",
                  d.source,
                  d.appId ? ` · ${d.appId}` : ""
                ] })
              ] }),
              /* @__PURE__ */ a("span", { className: `workbench-computer-pill ${d.status}`, children: d.status })
            ] }),
            /* @__PURE__ */ a("p", { className: "workbench-muted", children: d.description }),
            d.triggers?.length ? /* @__PURE__ */ a("div", { className: "workbench-computer-triggers", children: d.triggers.slice(0, 4).map((w) => /* @__PURE__ */ a("span", { children: w }, w)) }) : null,
            /* @__PURE__ */ a("button", { className: "workbench-button-subtle", onClick: () => {
              n(d);
            }, disabled: d.status === "missing", title: "Insert this skill's instructions into chat so Atomek can use the app/tool correctly.", children: "Use in chat" })
          ] }, d.id))
        ] })
      ] }) : null
    ] })
  ] });
}
function Ml({ status: e, file: t, cursor: n, fileCount: r, dirtyCount: i }) {
  const s = Ke(), o = e === "Ready" ? s("app.ready") : e;
  return /* @__PURE__ */ u("footer", { className: "workbench-statusbar", children: [
    /* @__PURE__ */ a("span", { children: "main" }),
    /* @__PURE__ */ a("span", { children: s("status.files", { count: r }) }),
    i > 0 && /* @__PURE__ */ a("span", { children: s("status.unsaved", { count: i }) }),
    /* @__PURE__ */ a("span", { className: "workbench-status-spacer" }),
    /* @__PURE__ */ a("span", { children: o }),
    /* @__PURE__ */ a("span", { children: s("status.lineColumn", { line: n.lineNumber, column: n.column }) }),
    /* @__PURE__ */ a("span", { children: s("status.spaces") }),
    /* @__PURE__ */ a("span", { children: "UTF-8" }),
    /* @__PURE__ */ a("span", { children: "LF" }),
    /* @__PURE__ */ a("span", { children: $a(t.language) })
  ] });
}
function jn(e, t) {
  return e.length === 0 ? !0 : window.confirm(`${e.length} file${e.length === 1 ? "" : "s"} have unsaved changes. Continue to ${t}?`);
}
function Jt(e, t) {
  const n = new Map(e.map((r) => [r.id, r]));
  return t.forEach((r) => n.set(r.id, r)), Array.from(n.values());
}
function Mn(e) {
  return e.replace(/\.[a-z0-9]+$/i, "").toLowerCase().replace(/[^a-z0-9]+/g, "-").replace(/^-+|-+$/g, "").slice(0, 54) || "ai-artifact";
}
function Wn(e, t) {
  const n = new Set(e.map((s) => s.path));
  let r = `${t}.md`, i = 2;
  for (; n.has(r); )
    r = `${t}-${i}.md`, i += 1;
  return r;
}
function Wl(e) {
  return {
    fileId: e.fileId,
    fileName: e.filePath,
    originalContent: e.originalContent,
    proposedContent: e.proposedContent,
    sourceTitle: e.sourceTitle,
    extractionLabel: e.extractionLabel,
    stats: e.stats
  };
}
function Vl(e) {
  const t = e.split(`
`);
  return t.slice(0, 80).join(`
`) + (t.length > 80 ? `
…` : "");
}
function Il() {
  try {
    const e = localStorage.getItem(Lr);
    if (!e) return [];
    const t = JSON.parse(e);
    return Yr(t);
  } catch {
    return [];
  }
}
function Yr(e) {
  return Array.isArray(e) ? e.filter((t) => !!t && typeof t.name == "string" && typeof t.path == "string").map((t) => ({
    name: t.name,
    path: t.path,
    at: typeof t.at == "number" ? t.at : 0,
    kind: t.kind === "folder" ? "folder" : "file",
    handleKey: typeof t.handleKey == "string" ? t.handleKey : void 0
  })).sort((t, n) => n.at - t.at).slice(0, 10) : [];
}
function Fl(...e) {
  const t = /* @__PURE__ */ new Map();
  for (const n of e.flatMap(Yr)) {
    const r = `${n.kind ?? "file"}:${n.handleKey ?? n.path}`, i = t.get(r);
    (!i || n.at >= i.at) && t.set(r, n);
  }
  return [...t.values()].sort((n, r) => r.at - n.at).slice(0, 10);
}
function Dl() {
  try {
    const e = localStorage.getItem(Cr);
    if (!e) return {};
    const t = JSON.parse(e);
    return t && typeof t == "object" ? t : {};
  } catch {
    return {};
  }
}
function Rl(e) {
  return Array.isArray(e) ? e.filter((t) => t && typeof t.id == "string" && typeof t.path == "string" && typeof t.content == "string").map((t) => ({
    id: t.id,
    name: t.name,
    path: t.path,
    language: t.language,
    content: t.content,
    dirty: !!t.dirty,
    size: t.size,
    source: t.source
  })) : [];
}
function El(e) {
  let t = 0;
  const n = 2e6;
  return e.slice(0, 160).map((r) => {
    const s = t + r.content.length <= n ? r.content : "";
    return t += s.length, {
      id: r.id,
      name: r.name,
      path: r.path,
      language: r.language,
      content: s,
      dirty: r.dirty,
      size: r.size,
      source: r.source
    };
  });
}
function ql(e) {
  try {
    localStorage.setItem(Cr, JSON.stringify(e));
  } catch {
  }
}
function Yl(e, t) {
  const n = new Map(e.map((r) => [r.id, r]));
  return t.map((r) => {
    const i = n.get(r.id);
    return i && i.dirty ? { ...r, content: i.content, dirty: !0 } : r;
  });
}
function Jl() {
  const e = {
    primaryVisible: !0,
    primaryWidth: 300,
    secondaryVisible: !0,
    secondaryWidth: 520,
    markdownPreviewVisible: !0
  };
  try {
    const t = localStorage.getItem(Xr);
    if (!t) return e;
    const n = JSON.parse(t);
    return {
      primaryVisible: typeof n.primaryVisible == "boolean" ? n.primaryVisible : e.primaryVisible,
      primaryWidth: typeof n.primaryWidth == "number" ? Math.max(240, Math.min(460, n.primaryWidth)) : e.primaryWidth,
      secondaryVisible: typeof n.secondaryVisible == "boolean" ? n.secondaryVisible : e.secondaryVisible,
      secondaryWidth: typeof n.secondaryWidth == "number" ? Math.max(380, Math.min(760, n.secondaryWidth)) : e.secondaryWidth,
      markdownPreviewVisible: typeof n.markdownPreviewVisible == "boolean" ? n.markdownPreviewVisible : e.markdownPreviewVisible
    };
  } catch {
    return e;
  }
}
function Zl() {
  try {
    const e = localStorage.getItem(Or);
    if (!e) return en;
    const t = JSON.parse(e);
    return {
      gatewayPreference: t.gatewayPreference === "remote" || t.gatewayPreference === "local" || t.gatewayPreference === "auto" ? t.gatewayPreference : en.gatewayPreference,
      model: typeof t.model == "string" ? t.model : "",
      embeddingModel: typeof t.embeddingModel == "string" ? t.embeddingModel : ""
    };
  } catch {
    return en;
  }
}
function Kl({ host: e }) {
  return /* @__PURE__ */ a(Wc, { host: e, children: /* @__PURE__ */ a(ol, { host: e }) });
}
function Ql(e) {
  return function() {
    return /* @__PURE__ */ a(Kl, { host: e.host });
  };
}
export {
  Ql as default
};
//# sourceMappingURL=index.js.map
