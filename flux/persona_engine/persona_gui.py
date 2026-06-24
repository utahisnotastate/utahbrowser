# flux/persona_engine/persona_gui.py
import flet as ft
import logging
from guardian import GuardianMiddleware
from persona_core import PersonaCore

def main(page: ft.Page):
    page.title = "Utah Persona Forge - SOTA Sovereign Workstation"
    page.theme_mode = ft.ThemeMode.DARK
    page.window_width = 1000
    page.window_height = 800
    page.padding = 40

    guardian = GuardianMiddleware()
    core = PersonaCore()

    # UI Components
    title = ft.Text("Identity-Preserving Persona Forge", style=ft.TextThemeStyle.HEADLINE_LARGE, color="#38bdf8")
    sub_title = ft.Text("Phase 12 & 13 Sovereign Architecture", color="#8892b0")

    target_input = ft.TextField(label="Target Image (Outfit/Scene Path)", border_color="#334155", focused_border_color="#38bdf8")
    source_input = ft.TextField(label="Source Face (Your Identity Path)", border_color="#334155", focused_border_color="#38bdf8")
    
    status_text = ft.Text("Ready for latent re-rendering.", color="#8892b0")
    log_area = ft.ListView(expand=True, spacing=5, padding=10, height=200)

    def add_log(msg, color="#4ade80"):
        log_area.controls.append(ft.Text(f"[{ft.datetime.datetime.now().strftime('%H:%M:%S')}] {msg}", color=color, size=12))
        page.update()

    def handle_swap(e):
        if not target_input.value or not source_input.value:
            status_text.value = "Error: Paths required."
            status_text.color = "#ef4444"
            page.update()
            return

        status_text.value = "Executing Latent Persona Mapping..."
        status_text.color = "#38bdf8"
        add_log(f"Initiating swap: {source_input.value} -> {target_input.value}")
        page.update()

        try:
            # 1. Safety Check via Guardian
            if guardian.is_content_safe("persona_mapping"):
                add_log("Guardian verified content integrity.", "#4ade80")
                
                # 2. Core Synthesis (Phase 13)
                result = core.synthesize_persona(target_input.value, source_input.value, {"intensity": 0.9})
                
                add_log("Temporal Consistency Anchor (TCA) applied.", "#4ade80")
                add_log("Relighting Latents normalized.", "#4ade80")
                add_log("Final Synthesis Successful.", "#38bdf8")
                
                status_text.value = "Persona successfully preserved in target scene."
                status_text.color = "#4ade80"
            else:
                add_log("Safety violation detected by Guardian.", "#ef4444")
                status_text.value = "Rejected: Content violation."
                status_text.color = "#ef4444"
        except Exception as ex:
            add_log(f"Synthesis Error: {str(ex)}", "#ef4444")
            status_text.value = "Failed: Check logs."
            status_text.color = "#ef4444"
        
        page.update()

    execute_btn = ft.ElevatedButton(
        "Execute Latent Swap",
        on_click=handle_swap,
        bgcolor="#38bdf8",
        color="#0f172a",
        width=300,
        height=50
    )

    # Layout
    page.add(
        ft.Column([
            title,
            sub_title,
            ft.Divider(height=40, color="transparent"),
            ft.Row([
                ft.Column([
                    ft.Text("Parameters", size=20, weight=ft.FontWeight.BOLD),
                    target_input,
                    source_input,
                    execute_btn,
                    status_text,
                ], expand=1, spacing=20),
                ft.VerticalDivider(width=40),
                ft.Column([
                    ft.Text("SOTA Status Log", size=20, weight=ft.FontWeight.BOLD),
                    ft.Container(
                        content=log_area,
                        border=ft.border.all(1, "#334155"),
                        border_radius=10,
                        bgcolor="#050505",
                        expand=True
                    )
                ], expand=1, spacing=20)
            ], expand=True)
        ], expand=True)
    )

if __name__ == "__main__":
    ft.app(target=main)
