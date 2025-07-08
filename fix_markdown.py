#!/usr/bin/env python3

import re
import sys

def fix_markdown_formatting(content):
    lines = content.split('\n')
    fixed_lines = []
    
    i = 0
    while i < len(lines):
        line = lines[i]
        
        # Add blank line before headings (if previous line is not empty)
        if re.match(r'^#{1,6}\s', line) and i > 0 and lines[i-1].strip() != '':
            fixed_lines.append('')
        
        # Remove trailing punctuation from headings
        if re.match(r'^#{1,6}\s', line):
            line = re.sub(r':$', '', line)
        
        fixed_lines.append(line)
        
        # Add blank line after headings (if next line is not empty)
        if re.match(r'^#{1,6}\s', line) and i+1 < len(lines) and lines[i+1].strip() != '':
            fixed_lines.append('')
        
        # Add blank lines around code blocks
        if re.match(r'^```', line):
            # Add blank line before code block
            if i > 0 and lines[i-1].strip() != '':
                fixed_lines.insert(-1, '')
            
            # Find end of code block
            j = i + 1
            while j < len(lines) and not re.match(r'^```\s*$', lines[j]):
                j += 1
            
            # Copy code block content
            for k in range(i+1, j+1):
                if k < len(lines):
                    fixed_lines.append(lines[k])
            
            # Add blank line after code block
            if j+1 < len(lines) and lines[j+1].strip() != '':
                fixed_lines.append('')
            
            i = j
        
        # Add blank lines around lists
        if re.match(r'^- ', line):
            # Add blank line before first list item
            if i > 0 and not re.match(r'^- ', lines[i-1]) and lines[i-1].strip() != '':
                fixed_lines.insert(-1, '')
            
            # Find end of list
            j = i
            while j+1 < len(lines) and (re.match(r'^- ', lines[j+1]) or re.match(r'^  ', lines[j+1])):
                j += 1
                fixed_lines.append(lines[j])
            
            # Add blank line after list
            if j+1 < len(lines) and lines[j+1].strip() != '':
                fixed_lines.append('')
            
            i = j
        
        i += 1
    
    # Remove consecutive empty lines
    result = []
    prev_empty = False
    for line in fixed_lines:
        if line.strip() == '':
            if not prev_empty:
                result.append(line)
            prev_empty = True
        else:
            result.append(line)
            prev_empty = False
    
    return '\n'.join(result)

if __name__ == '__main__':
    if len(sys.argv) != 2:
        print("Usage: python3 fix_markdown.py <file>")
        sys.exit(1)
    
    filename = sys.argv[1]
    
    with open(filename, 'r', encoding='utf-8') as f:
        content = f.read()
    
    fixed_content = fix_markdown_formatting(content)
    
    with open(filename, 'w', encoding='utf-8') as f:
        f.write(fixed_content)
    
    print(f"Fixed markdown formatting in {filename}")
