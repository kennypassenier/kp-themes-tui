// Snippet colouring that borrows every colour from the theme [T12, KT8].
//
// A third-party highlighter brings its own palette, and KT8's browser
// test fails a page that paints a colour the theme did not give it — so a
// library would have to be remapped onto tokens anyway, and the remapping
// is the larger half of the work. This is the smaller half, written here.
//
// Three token classes, and the mapping is the whole design:
//
//   kp-code__keyword   --code-keyword       names the language gives meaning to:
//                                           JS keywords, CSS property and at-rule
//                                           names, HTML tag and attribute names
//   kp-code__string    --code-string        values: strings, numbers, hex colours,
//                                           HTML attribute values
//   kp-code__comment   --muted-foreground   comments, in all three languages
//
// Everything else — punctuation, identifiers, text — carries no span and
// inherits `--foreground`. Three classes is not poverty: a snippet in
// documentation is read for its shape, and the two chart hues plus the
// muted ink are what every theme is guaranteed to declare.
//
// The stylesheet is emitted from the same table (`highlightCss()`), so a
// colour cannot enter through the CSS half either.
//
// Usage: node gates/site/highlight.mjs   (prints the stylesheet)

import process from 'node:process';

/** The mapping, in one place, because the CSS and the classes must agree. */
export const TOKEN_TOKENS = Object.freeze({
    keyword: '--code-keyword',
    string: '--code-string',
    comment: '--muted-foreground',
});

/** @typedef {'keyword' | 'string' | 'comment' | 'plain'} Kind */
/** @typedef {{ kind: Kind, text: string }} Token */

export const LANGUAGES = Object.freeze(['html', 'css', 'js']);

/** What a documentation snippet actually contains. Not the whole language. */
const JS_KEYWORDS = new Set([
    'as',
    'async',
    'await',
    'break',
    'case',
    'catch',
    'class',
    'const',
    'continue',
    'default',
    'delete',
    'do',
    'else',
    'export',
    'extends',
    'false',
    'finally',
    'for',
    'from',
    'function',
    'if',
    'import',
    'in',
    'instanceof',
    'let',
    'new',
    'null',
    'of',
    'return',
    'super',
    'switch',
    'this',
    'throw',
    'true',
    'try',
    'typeof',
    'undefined',
    'var',
    'void',
    'while',
    'yield',
]);

/**
 * @param {Token[]} out
 * @param {Kind} kind
 * @param {string} text
 */
function push(out, kind, text) {
    if (text === '') return;
    const last = out[out.length - 1];
    if (last && last.kind === kind) last.text += text;
    else out.push({ kind, text });
}

/** @param {string} code @returns {Token[]} */
function tokenizeJs(code) {
    /** @type {Token[]} */
    const out = [];
    let i = 0;
    while (i < code.length) {
        const rest = code.slice(i);
        const line = /^\/\/[^\n]*/.exec(rest);
        if (line) {
            push(out, 'comment', line[0]);
            i += line[0].length;
            continue;
        }
        const block = /^\/\*[\s\S]*?(?:\*\/|$)/.exec(rest);
        if (block) {
            push(out, 'comment', block[0]);
            i += block[0].length;
            continue;
        }
        const string = /^(['"`])(?:\\.|(?!\1)[\s\S])*\1?/.exec(rest);
        if (string) {
            push(out, 'string', string[0]);
            i += string[0].length;
            continue;
        }
        const number = /^\d[\d_.]*(?:e[+-]?\d+)?/i.exec(rest);
        if (number) {
            push(out, 'string', number[0]);
            i += number[0].length;
            continue;
        }
        const word = /^[A-Za-z_$][\w$]*/.exec(rest);
        if (word) {
            push(out, JS_KEYWORDS.has(word[0]) ? 'keyword' : 'plain', word[0]);
            i += word[0].length;
            continue;
        }
        push(out, 'plain', code[i] ?? '');
        i++;
    }
    return out;
}

/**
 * Looking forward from a `name:` to decide whether it was a property.
 *
 * @param {string} after the text following the identifier
 */
function endsAsDeclaration(after) {
    let parens = 0;
    for (const ch of after) {
        if (ch === '(') parens++;
        else if (ch === ')') {
            if (parens === 0) return true;
            parens--;
        } else if (parens === 0 && (ch === ';' || ch === '}')) return true;
        else if (parens === 0 && ch === '{') return false;
    }
    return false;
}

/** @param {string} code @returns {Token[]} */
function tokenizeCss(code) {
    /** @type {Token[]} */
    const out = [];
    let i = 0;
    let depth = 0;
    // A property name is an identifier at the start of a declaration, so
    // `a:hover` in a selector is not one and `color:` is.
    let atStatementStart = true;
    while (i < code.length) {
        const rest = code.slice(i);
        const comment = /^\/\*[\s\S]*?(?:\*\/|$)/.exec(rest);
        if (comment) {
            push(out, 'comment', comment[0]);
            i += comment[0].length;
            continue;
        }
        const string = /^(['"])(?:\\.|(?!\1)[^\n])*\1?/.exec(rest);
        if (string) {
            push(out, 'string', string[0]);
            i += string[0].length;
            continue;
        }
        const at = /^@[\w-]+/.exec(rest);
        if (at) {
            push(out, 'keyword', at[0]);
            i += at[0].length;
            atStatementStart = false;
            continue;
        }
        const hex = /^#[0-9a-f]{3,8}\b/i.exec(rest);
        if (hex) {
            push(out, 'string', hex[0]);
            i += hex[0].length;
            continue;
        }
        const number = /^\d[\d.]*(?:%|[a-z]+)?/i.exec(rest);
        if (number) {
            push(out, 'string', number[0]);
            i += number[0].length;
            continue;
        }
        const ident = /^--?[\w-]+|^[A-Za-z][\w-]*/.exec(rest);
        if (ident) {
            // `color:` inside a block is a property and `min-width:` in a
            // media prelude is one too; `a:hover` and `a:not(.x)` in a
            // selector are not. What separates them is what comes next: a
            // declaration reaches a `;` — or the end of the prelude's
            // parenthesis — before it reaches a `{`.
            const after = rest.slice(ident[0].length);
            const declaration = atStatementStart && /^\s*:/.test(after) && endsAsDeclaration(after);
            push(out, declaration || (atStatementStart && ident[0].startsWith('--')) ? 'keyword' : 'plain', ident[0]);
            i += ident[0].length;
            atStatementStart = false;
            continue;
        }
        const ch = code[i] ?? '';
        if (ch === '{') depth++;
        if (ch === '}') depth = Math.max(0, depth - 1);
        if (ch === '{' || ch === '}' || ch === ';' || ch === '(' || ch === ',') atStatementStart = true;
        else if (!/\s/.test(ch)) atStatementStart = false;
        push(out, 'plain', ch);
        i++;
    }
    return out;
}

/** @param {string} code @returns {Token[]} */
function tokenizeHtml(code) {
    /** @type {Token[]} */
    const out = [];
    let i = 0;
    while (i < code.length) {
        const rest = code.slice(i);
        const comment = /^<!--[\s\S]*?(?:-->|$)/.exec(rest);
        if (comment) {
            push(out, 'comment', comment[0]);
            i += comment[0].length;
            continue;
        }
        const open = /^<\/?[A-Za-z][\w-]*/.exec(rest);
        if (open) {
            const slash = open[0].indexOf('/') === 1 ? 2 : 1;
            push(out, 'plain', open[0].slice(0, slash));
            push(out, 'keyword', open[0].slice(slash));
            i += open[0].length;
            // Inside the tag: attribute names are keywords, values strings.
            while (i < code.length && code[i] !== '>') {
                const inside = code.slice(i);
                const name = /^[\s]*[A-Za-z:@][\w:.-]*/.exec(inside);
                const value = /^\s*=\s*(?:"[^"]*"?|'[^']*'?|[^\s>]+)/.exec(inside);
                if (value) {
                    push(out, 'string', value[0]);
                    i += value[0].length;
                    continue;
                }
                if (name) {
                    const lead = /^\s*/.exec(name[0])?.[0] ?? '';
                    push(out, 'plain', lead);
                    push(out, 'keyword', name[0].slice(lead.length));
                    i += name[0].length;
                    continue;
                }
                push(out, 'plain', code[i] ?? '');
                i++;
            }
            continue;
        }
        push(out, 'plain', code[i] ?? '');
        i++;
    }
    return out;
}

/**
 * @param {string} code
 * @param {string} language one of LANGUAGES
 * @returns {Token[]}
 */
export function tokenize(code, language) {
    if (language === 'js') return tokenizeJs(code);
    if (language === 'css') return tokenizeCss(code);
    if (language === 'html') return tokenizeHtml(code);
    // The same refusal the renderer makes: a language nobody taught this
    // thing would otherwise be shipped as flat, unlabelled text.
    throw new Error(`highlight: no tokenizer for "${language}". Known: ${LANGUAGES.join(', ')}.`);
}

/** @param {string} text */
function escapeHtml(text) {
    return text.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;');
}

/**
 * @param {string} code
 * @param {string} language
 * @returns {string} the snippet's inner HTML, spans and escaped text
 */
export function highlight(code, language) {
    return tokenize(code, language)
        .map((token) => (token.kind === 'plain' ? escapeHtml(token.text) : `<span class="kp-code__${token.kind}">${escapeHtml(token.text)}</span>`))
        .join('');
}

/**
 * The stylesheet, generated from the same table as the classes.
 *
 * Written here rather than in a hand-kept CSS file so the promise "no
 * colour of its own" is structural: there is no place to put one.
 */
export function highlightCss() {
    const rules = Object.entries(TOKEN_TOKENS)
        .map(([kind, token]) => `.kp-code__${kind} {\n    color: var(${token});\n}`)
        .join('\n\n');
    return `${rules}\n`;
}

if (import.meta.url === `file://${process.argv[1]}`) {
    // AR26: the expectation is the mapping table, not whatever the
    // tokenizer happened to emit.
    const kinds = Object.keys(TOKEN_TOKENS);
    const css = highlightCss();
    const missing = kinds.filter((kind) => !css.includes(`.kp-code__${kind}`));
    if (missing.length > 0) {
        console.error(`Highlighter: ${missing.length} of ${kinds.length} token classes have no rule: ${missing.join(', ')}.`);
        process.exit(1);
    }
    process.stdout.write(css);
    console.log(`Highlighter: ${kinds.length} token classes, ${LANGUAGES.length} languages, every colour from a token.`);
}
