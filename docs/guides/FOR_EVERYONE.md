# Utah Browser: The Easy Guide to Sovereign Browsing

**For parents, students, professionals, and anyone who values their digital freedom.**

Utah Browser is not just a replacement for Chrome or Edge—it is a private workstation that works for *you*, not for advertisers. This guide will show you how to download, install, and master your new sovereign space.

---

## 1. What makes Utah Browser different?

Traditional browsers watch what you do so they can sell ads. Utah Browser does the opposite:
- **Privacy by Default:** Your history and files stay on your computer.
- **Truth Guard AI:** An AI assistant that answers questions using *your* notes and history—never sending your data to the cloud.
- **Ad-Free Forever:** We don't just block ads; we "ablate" them, removing the very space they take up on a page.
- **Fast & Light:** Inactive tabs are "put to sleep" to save your computer's memory.

---

## 2. Getting Started (The 5-Minute Setup)

### Step A: The Ingredients
You only need two things installed on your PC first:
1. **Ollama:** This is the "brain" for your local AI. Download it at [ollama.com](https://ollama.com/).
2. **Rust:** This is the engine that runs the browser. Download it at [rustup.rs](https://rustup.rs/).

### Step B: The Installation
If you have the Utah Browser folder on your computer, follow these simple steps:
1. Open a **PowerShell** window (click Start, type "PowerShell", and press Enter).
2. Type `cd C:\code\utahbrowser` (or wherever you saved the folder) and press Enter.
3. Type `.\scripts\install.ps1` and press Enter.

*The browser will now set itself up, download its search database (Qdrant), and build your application. This usually takes 2-3 minutes.*

### Step C: Launch!
Once the installation finishes, type:
```powershell
cd dist
.\UtahBrowser.cmd
```

---

## 3. Mastering the SOTA Features

### The Sovereign History Oracle (Truth Guard)
Want to find that recipe you saw last week or a PDF you downloaded? Just click the **Chat** icon. You can ask: *"What was that article about solar panels I read?"* or *"Summarize my tax PDF from yesterday."* The AI reads your *local* files to give you an instant answer.

### Seamless Privacy (.tor)
If you visit a `.onion` website or want maximum privacy, Utah Browser automatically handles the "secret tunnels" (Tor) for you. You don't have to change any settings; it just works.

### The Email Nexus
Manage your emails without being tracked. Our Email Nexus strips out "tracking pixels" that companies use to spy on when you open their mail. It's a clean, safe, and fast way to stay connected.

### Career Forge (Job Automation)
Applying for jobs? Career Forge can take your resume and automatically tailor it to a specific job description, highlighting the skills the employer is looking for. It even keeps a private log of every job you've applied to.

### Persona Forge (Virtual Stylist)
The new Persona Forge allows you to safely "try on" clothes or place yourself in new scenes. By using local AI, it maps your face onto a target image with perfect lighting and zero jitter. Because it runs on your PC, your photos never leave your machine.

### Sovereign Secure Shield (Ad-Free Browsing)
Stay safe on any website. The **Secure Shield** blocks aggressive ads, popups, and tracking scripts before they even load. You can see how many threats have been blocked by clicking the **Shield** icon in the browser toolbar, which opens a real-time dashboard of your protection status.

### Fluidic UI (The "Cool" Factor)
You'll notice the buttons and panels tilt and glow as you move your mouse. This is our **Fluidic Spatial Topography**. It makes the browser feel like a physical object made of glass. If you find it distracting, you can turn it off in the **Settings** menu.

---

## 4. Troubleshooting (When things act weird)

- **AI is slow or not answering:** Make sure **Ollama** is running (check your system tray near the clock).
- **Search isn't working:** Run the installer again (`.\scripts\install.ps1`) to make sure the database is healthy.
- **The screen is blank:** Ensure you have the latest Windows updates, as Utah Browser uses the built-in Windows web engine (WebView2).

---

**Welcome to the Future.** You are no longer the product; you are the Sovereign.

[Back to Documentation Hub](../README.md)
