with open('client/src/components/terminal/CrtTerminal.svelte', 'r', encoding='utf-8') as f:
    content = f.read()

# 1. Title size
content = content.replace(
    'class="text-xl sm:text-2xl font-black font-pixel-title tracking-wider drop-shadow-[0_0_12px_rgba(0,240,255,0.8)] flex items-center gap-2"',
    'class="text-base sm:text-lg font-bold font-pixel-title tracking-wider drop-shadow-[0_0_10px_rgba(0,240,255,0.7)] flex items-center gap-2"'
)

# 2. Clock size
content = content.replace(
    '<span class="font-pixel-terminal text-base">{clockTime}</span>',
    '<span class="font-pixel-terminal text-sm">{clockTime}</span>'
)

# 3. Progress percent
content = content.replace(
    '<span class="font-pixel-terminal text-base font-bold" style="color: {currentDroid?.accentColor || \'#00f0ff\'}">',
    '<span class="font-pixel-terminal text-sm font-bold" style="color: {currentDroid?.accentColor || \'#00f0ff\'}">'
)

# 4. Metrics: Tokens in, Tokens out, Latency
content = content.replace(
    '<span class="font-bold text-cyan-400 font-pixel-terminal text-base">{$activeProject.metrics.tokensIn.toLocaleString()}</span>',
    '<span class="font-bold text-cyan-400 font-pixel-terminal text-sm">{$activeProject.metrics.tokensIn.toLocaleString()}</span>'
)
content = content.replace(
    '<span class="font-bold text-fuchsia-400 font-pixel-terminal text-base">{$activeProject.metrics.tokensOut.toLocaleString()}</span>',
    '<span class="font-bold text-fuchsia-400 font-pixel-terminal text-sm">{$activeProject.metrics.tokensOut.toLocaleString()}</span>'
)
content = content.replace(
    '<span class="font-bold text-emerald-400 font-pixel-terminal text-base">{$activeProject.metrics.latencyMs}ms</span>',
    '<span class="font-bold text-emerald-400 font-pixel-terminal text-sm">{$activeProject.metrics.latencyMs}ms</span>'
)

# 5. User dialog bubble
content = content.replace(
    'max-w-[85%] sm:max-w-[70%] p-3.5 rounded-2xl bg-emerald-950/70 border-2 border-emerald-400/90 shadow-[0_0_20px_rgba(52,211,153,0.35)] text-emerald-200',
    'max-w-[85%] sm:max-w-[70%] p-2.5 sm:p-3 rounded-xl bg-emerald-950/70 border-2 border-emerald-400/90 shadow-[0_0_15px_rgba(52,211,153,0.3)] text-emerald-200'
)
content = content.replace(
    '<span class="text-emerald-500/70 text-[9px] font-pixel-terminal text-sm">{msg.timestamp}</span>',
    '<span class="text-emerald-500/70 text-[8.5px] font-pixel-terminal text-xs">{msg.timestamp}</span>'
)
content = content.replace(
    'class="font-pixel-terminal text-lg sm:text-xl text-emerald-100 font-medium tracking-wide"',
    'class="font-pixel-terminal text-sm sm:text-base text-emerald-100 font-normal leading-snug tracking-wide"'
)

# 6. Agent dialog bubble
content = content.replace(
    'max-w-[90%] sm:max-w-[78%] p-3.5 rounded-2xl {accentBg} border-2 {accentBorder} {accentShadow} text-slate-100',
    'max-w-[90%] sm:max-w-[78%] p-2.5 sm:p-3 rounded-xl {accentBg} border-2 {accentBorder} {accentShadow} text-slate-100'
)
content = content.replace(
    '<span class="opacity-70 text-[9px] font-pixel-terminal text-sm">[{msg.timestamp}]</span>',
    '<span class="opacity-70 text-[8.5px] font-pixel-terminal text-xs">[{msg.timestamp}]</span>'
)
content = content.replace(
    'class="font-pixel-terminal text-lg sm:text-xl text-slate-100 font-medium leading-relaxed tracking-wide whitespace-pre-wrap"',
    'class="font-pixel-terminal text-sm sm:text-base text-slate-100 font-normal leading-snug tracking-wide whitespace-pre-wrap"'
)
content = content.replace(
    '<span class="inline-block w-2.5 h-4 ml-1 bg-cyan-400 animate-pulse align-middle"></span>',
    '<span class="inline-block w-1.5 h-3.5 ml-1 bg-cyan-400 animate-pulse align-middle"></span>'
)

# 7. Chain of Thought
content = content.replace(
    'class="mt-2.5 p-2 rounded-lg bg-black/60 border border-slate-700/60 text-slate-300 font-pixel-terminal text-base"',
    'class="mt-2 p-2 rounded-lg bg-black/60 border border-slate-700/60 text-slate-300 font-pixel-terminal text-xs sm:text-sm"'
)

# 8. System message
content = content.replace(
    'class="text-center my-2 text-[11px] text-slate-400 font-pixel-terminal text-base"',
    'class="text-center my-1.5 text-slate-400 font-pixel-terminal text-xs"'
)

# 9. CLI prompt & input
content = content.replace(
    'class="relative flex items-center gap-2 sm:gap-3 p-2.5 rounded-2xl bg-slate-950/95 border-2 border-cyan-400/80 shadow-[0_0_25px_rgba(6,182,212,0.45)]"',
    'class="relative flex items-center gap-2 sm:gap-2.5 p-2 sm:p-2.5 rounded-xl bg-slate-950/95 border-2 border-cyan-400/80 shadow-[0_0_20px_rgba(6,182,212,0.4)]"'
)
content = content.replace(
    'class="flex items-center gap-1.5 font-pixel-terminal text-cyan-400 font-black text-2xl pl-2 select-none"',
    'class="flex items-center gap-1 font-pixel-terminal text-cyan-400 font-bold text-base sm:text-lg pl-2 select-none"'
)
content = content.replace(
    'class="w-full bg-transparent text-cyan-100 placeholder:text-slate-500 font-pixel-terminal text-xl sm:text-2xl focus:outline-none transition leading-normal pl-1 tracking-wide"',
    'class="w-full bg-transparent text-cyan-100 placeholder:text-slate-500 font-pixel-terminal text-sm sm:text-base focus:outline-none transition leading-normal pl-1 tracking-wide"'
)
content = content.replace(
    '<span class="w-2.5 h-5 bg-cyan-400 animate-pulse ml-0.5 pointer-events-none"></span>',
    '<span class="w-1.5 h-3.5 bg-cyan-400 animate-pulse ml-0.5 pointer-events-none"></span>'
)

with open('client/src/components/terminal/CrtTerminal.svelte', 'w', encoding='utf-8') as f:
    f.write(content)

print("SUCCESS: CrtTerminal font sizes refined!")
