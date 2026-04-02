import os
import glob

folder = r'c:\Users\rajku\Downloads\sem 6\AC applied cryptography\projects\PROJECT_SENTINEL_V4\SENTINEL_CBDC_PROJECT\frontend\admin\*.html'
files = glob.glob(folder)

LINK = '''<a href="defrag.html" class="flex items-center gap-3 px-4 py-3 hover:bg-slate-800 text-purple-400 hover:text-purple-300 rounded-xl transition">
                <span>🔧</span> Defrag & Transactions
            </a>\n        </nav>'''
            
for f in files:
    if 'defrag.html' in f: continue
    if 'dashboard.html' in f: continue
    with open(f, 'r', encoding='utf-8') as file:
        content = file.read()
    if 'Defrag & Transactions' not in content:
        if '</nav>' in content:
            content = content.replace('</nav>', LINK)
            with open(f, 'w', encoding='utf-8') as file:
                file.write(content)
            print('Updated', f)
        else:
            print('Could not find anchor in', f)
