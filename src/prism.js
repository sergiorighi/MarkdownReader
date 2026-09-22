/* PrismJS 1.29.0 - Lightweight Bundle for MarkdownReader */
(function(global) {
    var langPattern = /\blang(?:uage)?-([\w-]+)\b/i;
    var uniqueId = 0;

    var Prism = {
        manual: true,
        util: {
            encode: function(tokens) {
                if (tokens instanceof Token) {
                    return new Token(tokens.type, Prism.util.encode(tokens.content), tokens.alias);
                } else if (Array.isArray(tokens)) {
                    return tokens.map(Prism.util.encode);
                } else {
                    return tokens.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/\u00a0/g, ' ');
                }
            },
            type: function(o) {
                return Object.prototype.toString.call(o).slice(8, -1);
            },
            objId: function(obj) {
                if (!obj['__id']) {
                    Object.defineProperty(obj, '__id', { value: ++uniqueId });
                }
                return obj['__id'];
            },
            clone: function deepClone(o, visited) {
                visited = visited || {};
                var clone, id;
                switch (Prism.util.type(o)) {
                    case 'Object':
                        id = Prism.util.objId(o);
                        if (visited[id]) return visited[id];
                        clone = {};
                        visited[id] = clone;
                        for (var key in o) {
                            if (o.hasOwnProperty(key)) clone[key] = deepClone(o[key], visited);
                        }
                        return clone;
                    case 'Array':
                        id = Prism.util.objId(o);
                        if (visited[id]) return visited[id];
                        clone = [];
                        visited[id] = clone;
                        o.forEach(function(v, i) { clone[i] = deepClone(v, visited); });
                        return clone;
                    default:
                        return o;
                }
            },
            getLanguage: function(element) {
                while (element) {
                    var match = langPattern.exec(element.className);
                    if (match) return match[1].toLowerCase();
                    element = element.parentElement;
                }
                return 'none';
            },
            setLanguage: function(element, language) {
                element.className = element.className.replace(RegExp(langPattern, 'gi'), '');
                element.classList.add('language-' + language);
            }
        },
        languages: {
            plain: {},
            plaintext: {},
            text: {},
            txt: {},
            extend: function(id, redef) {
                var lang = Prism.util.clone(Prism.languages[id]);
                for (var key in redef) {
                    lang[key] = redef[key];
                }
                return lang;
            },
            insertBefore: function(inside, before, insert, root) {
                root = root || Prism.languages;
                var grammar = root[inside];
                var ret = {};
                for (var item in grammar) {
                    if (grammar.hasOwnProperty(item)) {
                        if (item === before) {
                            for (var newNode in insert) {
                                if (insert.hasOwnProperty(newNode)) ret[newNode] = insert[newNode];
                            }
                        }
                        if (!insert.hasOwnProperty(item)) ret[item] = grammar[item];
                    }
                }
                var old = root[inside];
                root[inside] = ret;
                Prism.languages.DFS(Prism.languages, function(key, value) {
                    if (value === old && key !== inside) this[key] = ret;
                });
                return ret;
            },
            DFS: function DFS(obj, callback, type, visited) {
                visited = visited || {};
                var objId = Prism.util.objId;
                for (var i in obj) {
                    if (obj.hasOwnProperty(i)) {
                        callback.call(obj, i, obj[i], type || i);
                        var property = obj[i],
                            propertyType = Prism.util.type(property);
                        if (propertyType === 'Object' && !visited[objId(property)]) {
                            visited[objId(property)] = true;
                            DFS(property, callback, null, visited);
                        } else if (propertyType === 'Array' && !visited[objId(property)]) {
                            visited[objId(property)] = true;
                            DFS(property, callback, i, visited);
                        }
                    }
                }
            }
        },
        highlightElement: function(element, async, callback) {
            var language = Prism.util.getLanguage(element);
            var grammar = Prism.languages[language];
            Prism.util.setLanguage(element, language);

            var code = element.textContent;
            if (!grammar) {
                element.innerHTML = Prism.util.encode(code);
                if (callback) callback.call(element);
                return;
            }

            var highlighted = Prism.highlight(code, grammar, language);
            element.innerHTML = highlighted;
            if (callback) callback.call(element);
        },
        highlight: function(text, grammar, language) {
            var env = { code: text, grammar: grammar, language: language };
            var tokens = Prism.tokenize(env.code, env.grammar);
            return Token.stringify(Prism.util.encode(tokens), env.language);
        },
        tokenize: function(text, grammar) {
            var rest = grammar.rest;
            if (rest) {
                for (var token in rest) {
                    grammar[token] = rest[token];
                }
                delete grammar.rest;
            }

            var tokenList = new LinkedList();
            addAfter(tokenList, tokenList.head, text);

            matchGrammar(text, tokenList, grammar, tokenList.head, 0);

            return toArray(tokenList);
        }
    };

    function Token(type, content, alias, matchedStr) {
        this.type = type;
        this.content = content;
        this.alias = alias;
        this.length = (matchedStr || '').length | 0;
    }

    Token.stringify = function stringify(o, language) {
        if (typeof o === 'string') return o;
        if (Array.isArray(o)) {
            return o.map(function(element) {
                return stringify(element, language);
            }).join('');
        }

        var env = {
            type: o.type,
            content: stringify(o.content, language),
            tag: 'span',
            classes: ['token', o.type],
            attributes: {},
            language: language
        };

        var aliases = o.alias;
        if (aliases) {
            if (Array.isArray(aliases)) {
                Array.prototype.push.apply(env.classes, aliases);
            } else {
                env.classes.push(aliases);
            }
        }

        var attributes = '';
        for (var name in env.attributes) {
            attributes += ' ' + name + '="' + (env.attributes[name] || '').replace(/"/g, '&quot;') + '"';
        }

        return '<' + env.tag + ' class="' + env.classes.join(' ') + '"' + attributes + '>' + env.content + '</' + env.tag + '>';
    };

    function matchPattern(pattern, pos, text, lookbehind) {
        pattern.lastIndex = pos;
        var match = pattern.exec(text);
        if (match && lookbehind && match[1]) {
            var lookbehindLength = match[1].length;
            match.index += lookbehindLength;
            match[0] = match[0].slice(lookbehindLength);
        }
        return match;
    }

    function matchGrammar(text, tokenList, grammar, startNode, startPos, rematch) {
        for (var token in grammar) {
            if (!grammar.hasOwnProperty(token) || !grammar[token]) continue;

            var patterns = grammar[token];
            patterns = Array.isArray(patterns) ? patterns : [patterns];

            for (var j = 0; j < patterns.length; ++j) {
                if (rematch && rematch.cause === token + ',' + j) return;

                var patternObj = patterns[j],
                    inside = patternObj.inside,
                    lookbehind = !!patternObj.lookbehind,
                    greedy = !!patternObj.greedy,
                    alias = patternObj.alias;

                if (greedy && !patternObj.pattern.global) {
                    var flags = patternObj.pattern.toString().match(/[imsuy]*$/)[0];
                    patternObj.pattern = RegExp(patternObj.pattern.source, flags + 'g');
                }

                var pattern = patternObj.pattern || patternObj;

                for (var currentNode = startNode.next, pos = startPos; currentNode !== tokenList.tail; pos += currentNode.value.length, currentNode = currentNode.next) {
                    if (rematch && pos >= rematch.reach) return;

                    var str = currentNode.value;
                    if (tokenList.length > text.length) return;

                    if (str instanceof Token) continue;

                    var match, count = 1;

                    if (greedy) {
                        match = matchPattern(pattern, pos, text, lookbehind);
                        if (!match || match.index >= text.length) break;

                        var from = match.index,
                            to = match.index + match[0].length,
                            p = pos;

                        p += currentNode.value.length;
                        while (from >= p) {
                            currentNode = currentNode.next;
                            p += currentNode.value.length;
                        }
                        p -= currentNode.value.length;
                        pos = p;

                        if (currentNode.value instanceof Token) continue;

                        for (var k = currentNode; k !== tokenList.tail && (p < to || typeof k.value === 'string'); k = k.next) {
                            count++;
                            p += k.value.length;
                        }
                        count--;
                        str = text.slice(pos, p);
                        match.index -= pos;
                    } else {
                        match = matchPattern(pattern, 0, str, lookbehind);
                        if (!match) continue;
                    }

                    var from = match.index,
                        matchStr = match[0],
                        before = str.slice(0, from),
                        after = str.slice(from + matchStr.length);

                    var reach = pos + str.length;
                    if (rematch && reach > rematch.reach) {
                        rematch.reach = reach;
                    }

                    var removeFrom = currentNode.prev;

                    if (before) {
                        removeFrom = addAfter(tokenList, removeFrom, before);
                        pos += before.length;
                    }

                    removeRange(tokenList, removeFrom, count);

                    var wrapped = new Token(token, inside ? Prism.tokenize(matchStr, inside) : matchStr, alias, matchStr);
                    currentNode = addAfter(tokenList, removeFrom, wrapped);

                    if (after) {
                        addAfter(tokenList, currentNode, after);
                    }

                    if (count > 1) {
                        matchGrammar(text, tokenList, grammar, currentNode.prev, pos, {
                            cause: token + ',' + j,
                            reach: reach
                        });
                    }
                }
            }
        }
    }

    function LinkedList() {
        var head = { value: null, prev: null, next: null };
        var tail = { value: null, prev: head, next: null };
        head.next = tail;
        this.head = head;
        this.tail = tail;
        this.length = 0;
    }

    function addAfter(list, node, value) {
        var next = node.next;
        var newNode = { value: value, prev: node, next: next };
        node.next = newNode;
        next.prev = newNode;
        list.length++;
        return newNode;
    }

    function removeRange(list, node, count) {
        var next = node.next;
        for (var i = 0; i < count && next !== list.tail; i++) {
            next = next.next;
        }
        node.next = next;
        next.prev = node;
        list.length -= count;
    }

    function toArray(list) {
        var array = [];
        var node = list.head.next;
        while (node !== list.tail) {
            array.push(node.value);
            node = node.next;
        }
        return array;
    }

    /* GRAMMARS */

    /* Markup (HTML, XML, SVG) */
    Prism.languages.markup = {
        'comment': { pattern: /<!--(?:(?!<!--)[\s\S])*?-->/, greedy: true },
        'prolog': { pattern: /<\?[\s\S]+?\?>/, greedy: true },
        'doctype': { pattern: /<!DOCTYPE(?:[^>"'[\]]|"[^"]*"|'[^']*')+(?:\[(?:[^<"'\]]|"[^"]*"|'[^']*'|<(?!!--)|<!--(?:(?!<!--)[\s\S])*?-->)*\]\s*)?>/i, greedy: true },
        'cdata': { pattern: /<!\[CDATA\[[\s\S]*?\]\]>/i, greedy: true },
        'tag': {
            pattern: /<\/?(?!\d)[^\s>/$<%]+(?:\s(?:\s*[^\s>/=]+(?:\s*=\s*(?:"[^"]*"|'[^']*'|[^\s'">=]+(?=[\s>]|$))|(?=[\s/>])))+)?\s*\/?>/,
            greedy: true,
            inside: {
                'tag': { pattern: /^<\/?[^\s>/]+/, inside: { 'punctuation': /^<\/?/ } },
                'attr-value': { pattern: /=\s*(?:"[^"]*"|'[^']*'|[^\s'">=]+)/, inside: { 'punctuation': [/^=/, /^["']|["']$/] } },
                'punctuation': /\/?>/,
                'attr-name': /[^\s>/]+/
            }
        },
        'entity': [{ pattern: /&[\da-z]{1,8};/i, alias: 'named-entity' }, /&#x?[\da-f]{1,8};/i]
    };
    Prism.languages.html = Prism.languages.markup;
    Prism.languages.xml = Prism.languages.markup;
    Prism.languages.svg = Prism.languages.markup;

    /* CSS */
    Prism.languages.css = {
        'comment': /\/\*[\s\S]*?\*\//,
        'atrule': { pattern: /@[\w-](?:[^;{\s]|\s+(?![\s{]))+?(?:;|(?=\s*\{))/, inside: { 'rule': /^@[\w-]+/ } },
        'url': { pattern: /\burl\(["']?(?:[^"'\r\n\\]|\\.)*?["']?\)/i, greedy: true },
        'selector': { pattern: /(?!\s)[^{}]*(?=\s*\{)/, inside: { 'punctuation': /[,+~>]/ } },
        'string': { pattern: /(["'])(?:\\(?:\r\n|[\s\S])|(?!\1)[^\\\r\n])*\1/, greedy: true },
        'property': { pattern: /(^|[^\w-])[-_a-z\xA0-\uFFFF][-\w\xA0-\uFFFF]*(?=\s*:)/i, lookbehind: true },
        'important': /!important\b/i,
        'function': { pattern: /(^|[0-9_\.\-\s(])(?![\d])[-_a-z0-9]+(?=\()/i, lookbehind: true },
        'punctuation': /[(){};:,]/
    };

    /* C-like */
    Prism.languages.clike = {
        'comment': [
            { pattern: /(^|[^\\])\/\*[\s\S]*?(?:\*\/|$)/, lookbehind: true, greedy: true },
            { pattern: /(^|[^\\:])\/\/.*/, lookbehind: true, greedy: true }
        ],
        'string': { pattern: /(["'])(?:\\(?:\r\n|[\s\S])|(?!\1)[^\\\r\n])*\1/, greedy: true },
        'class-name': { pattern: /(\b(?:class|interface|extends|implements|trait|instanceof|new)\s+|\bcatch\s+\()[\w.\\]+/i, lookbehind: true },
        'keyword': /\b(?:if|else|while|do|for|return|in|instanceof|function|new|try|throw|catch|finally|null|break|continue)\b/,
        'boolean': /\b(?:true|false)\b/,
        'function': /\b\w+(?=\()/,
        'number': /\b0x[\da-f]+\b|(?:\b\d+(?:\.\d*)?|\B\.\d+)(?:e[+-]?\d+)?/i,
        'operator': /[<>]=?|[!=]=?=?|--?|\+\+?|&&?|\|\|?|[?*/~^%]/,
        'punctuation': /[{}[\];(),.:]/
    };

    /* JavaScript */
    Prism.languages.javascript = Prism.languages.extend('clike', {
        'class-name': [
            Prism.languages.clike['class-name'],
            { pattern: /(^|[^$\w\xA0-\uFFFF])(?!\s)[_$A-Z\xA0-\uFFFF](?:(?!\s)[$\w\xA0-\uFFFF])*(?=\.(?:constructor|prototype))/, lookbehind: true }
        ],
        'keyword': [
            { pattern: /((?:^|\})\s*)catch\b/, lookbehind: true },
            { pattern: /(^|[^.]|\.\.\.\s*)\b(?:as|async|await|break|case|class|const|continue|debugger|default|delete|do|else|enum|export|extends|finally|for|from|function|get|if|implements|import|in|instanceof|interface|let|new|null|of|package|private|protected|public|return|set|static|super|switch|this|throw|try|typeof|undefined|var|void|while|with|yield)\b/, lookbehind: true }
        ],
        'function': /#?(?!\s)[_$a-zA-Z\xA0-\uFFFF](?:(?!\s)[$\w\xA0-\uFFFF])*(?=\s*\()/,
        'number': /\b0[xX][0-9a-fA-F]+\b|\b0[bB][01]+\b|\b0[oO][0-7]+\b|(?:\b\d+(?:\.\d*)?|\B\.\d+)(?:[eE][+-]?\d+)?/,
        'operator': /--|\+\+|\*\*=?|=>|&&=?|\|\|=?|[!=]==|<<=?|>>>?=?|[-+*/%&|^!=<>]=?|\.{3}|\?\?=?|\?\.?/
    });
    Prism.languages.js = Prism.languages.javascript;

    /* TypeScript */
    Prism.languages.typescript = Prism.languages.extend('javascript', {
        'class-name': { pattern: /(\b(?:class|extends|implements|instanceof|interface|new|type)\s+)(?!keyof\b)(?!\s)[_$a-zA-Z\xA0-\uFFFF](?:(?!\s)[$\w\xA0-\uFFFF])*/, lookbehind: true, greedy: true },
        'builtin': /\b(?:Array|Function|Promise|any|boolean|never|number|object|string|symbol|unknown|void)\b/
    });
    Prism.languages.ts = Prism.languages.typescript;

    /* Bash */
    Prism.languages.bash = {
        'comment': { pattern: /(^|[\s#])#.*/, lookbehind: true, greedy: true },
        'string': [{ pattern: /(["'])(?:\\[\s\S]|\$\([^)]+\)|\$(?!\()|`[^`]+`|(?!\1)[^\\`$])*\1/, greedy: true }],
        'variable': [/\$?\(\([^\)]+\)\)/, /\$\{[^}]+\}/, { pattern: /\$[a-zA-Z_0-9]+/, greedy: true }],
        'function': { pattern: /(^|[\s;|&])(?:alias|cat|cd|chmod|chown|clear|cp|curl|cut|date|df|du|echo|env|eval|exec|exit|export|find|git|grep|head|history|kill|less|ln|ls|man|mkdir|mv|nano|ps|pwd|rm|rmdir|sed|ssh|sudo|tail|tar|tee|touch|vi|vim|wget|which)\b/, lookbehind: true },
        'keyword': /\b(?:if|then|else|elif|fi|for|while|in|do|done|case|esac|function|return|select|until)\b/,
        'boolean': /\b(?:true|false)\b/,
        'number': /\b\d+\b/,
        'operator': /&&|\|\||[<>|&;=]/,
        'punctuation': /[{}[\]();,]/
    };
    Prism.languages.sh = Prism.languages.bash;
    Prism.languages.shell = Prism.languages.bash;
    Prism.languages.zsh = Prism.languages.bash;

    /* PowerShell */
    Prism.languages.powershell = {
        'comment': [
            { pattern: /(^|[^`])<#[\s\S]*?#>/, lookbehind: true, greedy: true },
            { pattern: /(^|[^`])#.*/, lookbehind: true, greedy: true }
        ],
        'string': [{ pattern: /(["'])(?:`[\s\S]|(?!\1)[^`])*\1/, greedy: true }],
        'variable': [/\$(?:\w+|(?=\{[^}]+\})|[$?^_-])/, /\$\[[^\]]+\]/],
        'function': [
            { pattern: /\b(?:Get|Set|New|Remove|Start|Stop|Restart|Test|Write|Select|Where|Invoke|Out|Format|Export|Import|ConvertTo|ConvertFrom)-[a-zA-Z0-9]+\b/i },
            { pattern: /\b[a-zA-Z_][a-zA-Z0-9_]*(?=\()/i }
        ],
        'keyword': /\b(?:Begin|Break|Catch|Continue|Data|Do|DynamicParam|Else|ElseIf|End|Exit|Filter|Finally|For|ForEach|From|Function|If|In|InlineScript|Hidden|Parallel|Param|Process|Return|Sequence|Switch|Throw|Trap|Try|Until|Using|While|Workflow)\b/i,
        'boolean': /\$(?:true|false|null)\b/i,
        'number': /\b0x[\da-f]+\b|(?:\b\d+(?:\.\d*)?|\B\.\d+)(?:e[+-]?\d+)?/i,
        'operator': /-(?:eq|ne|gt|ge|lt|le|like|notlike|match|notmatch|contains|notcontains|in|notin|replace|split|join|and|or|xor|not|is|isnot)\b/i,
        'punctuation': /[{}[\];(),.:]/
    };
    Prism.languages.ps1 = Prism.languages.powershell;
    Prism.languages.pwsh = Prism.languages.powershell;

    /* Python */
    Prism.languages.python = {
        'comment': { pattern: /(^|[^\\])#.*/, lookbehind: true, greedy: true },
        'string': { pattern: /(?:[rub]|rb|br)?(?:("""|''')[\s\S]*?\1|("|')(?:\\.|(?!\2)[^\\\r\n])*\2)/i, greedy: true },
        'decorator': { pattern: /(^\s*)@\w+(?:\.\w+)*/im, lookbehind: true, alias: 'function' },
        'keyword': /\b(?:and|as|assert|async|await|break|class|continue|def|del|elif|else|except|exec|finally|for|from|global|if|import|in|is|lambda|nonlocal|not|or|pass|print|raise|return|try|while|with|yield)\b/,
        'builtin': /\b(?:bool|bytearray|bytes|classmethod|complex|dict|float|frozenset|int|list|object|property|range|set|staticmethod|str|super|tuple|type)\b/,
        'boolean': /\b(?:True|False|None)\b/,
        'number': /\b0x[\da-f]+\b|(?:\b\d+(?:\.\d*)?|\B\.\d+)(?:e[+-]?\d+)?/i,
        'operator': /[-+%=]=?|!=|\*\*?=?|\/\/?=?|<[<=>]?|>[=>]?|[&|^~]/,
        'punctuation': /[{}[\];(),.:]/
    };
    Prism.languages.py = Prism.languages.python;

    /* Rust */
    Prism.languages.rust = {
        'comment': [
            { pattern: /(^|[^\\])\/\*[\s\S]*?(?:\*\/|$)/, lookbehind: true, greedy: true },
            { pattern: /(^|[^\\:])\/\/.*/, lookbehind: true, greedy: true }
        ],
        'string': { pattern: /b?"(?:\\.|[^\\\r\n"])*"|b?r(#*)"[\s\S]*?"\1/, greedy: true },
        'class-name': /\b[A-Z]\w*\b/,
        'keyword': /\b(?:Self|abstract|as|async|await|become|box|break|const|continue|crate|do|dyn|else|enum|extern|false|final|fn|for|if|impl|in|let|loop|macro|match|mod|move|mut|override|priv|pub|ref|return|self|static|struct|super|trait|true|try|type|typeof|unsafe|unsized|use|virtual|where|while|yield)\b/,
        'boolean': /\b(?:true|false)\b/,
        'function': /\b[a-z_]\w*(?=\s*(?:::\s*<.*>\s*)?\()/i,
        'number': /\b(?:0x[\da-fA-F_]+|0o[0-7_]+|0b[01_]+|(?:(?:\d[\d_]*(?:\.[\d_]+)?|\.[\d_]+)(?:[eE][+-]?[\d_]+)?))(?:[iu](?:8|16|32|64|128|size)|f32|f64)?\b/,
        'operator': /[-+*\/%!^]=?|=[=>]?|&[&=]?|\|[|=]?|<<?=?|>>?=?|[@?]/,
        'punctuation': /[{}[\];(),.:]/
    };
    Prism.languages.rs = Prism.languages.rust;

    /* JSON */
    Prism.languages.json = {
        'property': { pattern: /(^|[^\\])"(?:\\.|[^\\"\r\n])*"(?=\s*:)/, lookbehind: true, greedy: true },
        'string': { pattern: /(^|[^\\])"(?:\\.|[^\\"\r\n])*"(?!\s*:)/, lookbehind: true, greedy: true },
        'comment': { pattern: /\/\/.*|\/\*[\s\S]*?(?:\*\/|$)/, greedy: true },
        'number': /-?\b\d+(?:\.\d+)?(?:e[+-]?\d+)?\b/i,
        'punctuation': /[{}[\]:,]/,
        'boolean': /\b(?:true|false)\b/,
        'null': { pattern: /\bnull\b/, alias: 'keyword' }
    };

    /* YAML */
    Prism.languages.yaml = {
        'scalar': { pattern: /([\-:]\s*(?:![^\s]+[\s]+)?)(?:[^\s#:=]+(?:\s+[^\s#:=]+)*)(?=\s*(?:$|#))/, lookbehind: true },
        'comment': /#.*/,
        'key': { pattern: /(^\s*)(?:""|''|[^\s#:=]+)(?=\s*:\s)/m, lookbehind: true, alias: 'atrule' },
        'string': { pattern: /(["'])(?:\\.|(?!\1)[^\\\r\n])*\1/, greedy: true },
        'number': /[+-]?(?:0x[\da-f]+|0o[0-7]+|(?:\b\d+(?:\.\d*)?|\B\.\d+)(?:e[+-]?\d+)?)/i,
        'boolean': /\b(?:true|false|yes|no|on|off)\b/i,
        'punctuation': /---|[:[\]{},|>?*&]/
    };
    Prism.languages.yml = Prism.languages.yaml;

    /* SQL */
    Prism.languages.sql = {
        'comment': { pattern: /(^|[^\\])(?:\/\*[\s\S]*?\*\/|--.*?(\r?\n|$))/, lookbehind: true, greedy: true },
        'string': { pattern: /(^|[^@\\])('(?:''|\\.|[^\\'])*')/, lookbehind: true, greedy: true },
        'keyword': /\b(?:ADD|ALL|ALTER|AND|ANY|AS|ASC|AUTHORIZATION|BACKUP|BEGIN|BETWEEN|BREAK|BROWSE|BULK|BY|CASCADE|CASE|CHECK|CHECKPOINT|CLOSE|CLUSTERED|COALESCE|COLLATE|COLUMN|COMMIT|COMPUTE|CONSTRAINT|CONTAINS|CONTAINSTABLE|CONTINUE|CONVERT|CREATE|CROSS|CURRENT|CURRENT_DATE|CURRENT_TIME|CURRENT_TIMESTAMP|CURRENT_USER|CURSOR|DATABASE|DBCC|DEALLOCATE|DECLARE|DEFAULT|DELETE|DENY|DESC|DISK|DISTINCT|DISTRIBUTED|DOUBLE|DROP|DUMP|ELSE|END|ERRLVL|ESCAPE|EXCEPT|EXEC|EXECUTE|EXISTS|EXIT|EXTERNAL|FETCH|FILE|FILLFACTOR|FOR|FOREIGN|FREETEXT|FREETEXTTABLE|FROM|FULL|FUNCTION|GOTO|GRANT|GROUP|HAVING|HOLDLOCK|IDENTITY|IDENTITY_INSERT|IDENTITYCOL|IF|IN|INDEX|INNER|INSERT|INTERSECT|INTO|IS|JOIN|KEY|KILL|LEFT|LIKE|LINENO|LOAD|MERGE|NATIONAL|NOCHECK|NONCLUSTERED|NOT|NULL|NULLIF|OF|OFF|OFFSETS|ON|OPEN|OPENDATASOURCE|OPENQUERY|OPENROWSET|OPENXML|OPTION|OR|ORDER|OUTER|OVER|PERCENT|PIVOT|PLAN|PRECISION|PRIMARY|PRINT|PROC|PROCEDURE|PUBLIC|RAISERROR|READ|READTEXT|RECONFIGURE|REFERENCES|REPLICATION|RESTORE|RESTRICT|RETURN|REVERT|REVOKE|RIGHT|ROLLBACK|ROWCOUNT|ROWGUIDCOL|RULE|SAVE|SCHEMA|SECURITYAUDIT|SELECT|SEMANTICKEYPHRASETABLE|SEMANTICSIMILARITYDETAILSTABLE|SEMANTICSIMILARITYTABLE|SESSION_USER|SET|SETUSER|SHUTDOWN|SOME|STATISTICS|SYSTEM_USER|TABLE|TABLESAMPLE|TEXTSIZE|THEN|TO|TOP|TRAN|TRANSACTION|TRIGGER|TRUNCATE|TRY_CONVERT|TSEQUAL|UNION|UNIQUE|UNPIVOT|UPDATE|UPDATETEXT|USE|USER|VALUES|VARYING|VIEW|WAITFOR|WHEN|WHERE|WHILE|WITH|WITHIN|WRITETEXT)\b/i,
        'boolean': /\b(?:TRUE|FALSE|NULL)\b/i,
        'number': /\b0x[\da-f]+\b|(?:\b\d+(?:\.\d*)?|\B\.\d+)(?:e[+-]?\d+)?/i,
        'operator': /[-+*\/%^~=]|&&?|\|\|?|!=?|<[=>]?|>[=>]?/,
        'punctuation': /[()[\];:,.]/
    };

    /* C# */
    Prism.languages.csharp = Prism.languages.extend('clike', {
        'keyword': /\b(?:abstract|as|async|await|base|bool|break|byte|case|catch|char|checked|class|const|continue|decimal|default|delegate|do|double|else|enum|event|explicit|extern|false|finally|fixed|float|for|foreach|goto|if|implicit|in|int|interface|internal|is|lock|long|namespace|new|null|object|operator|out|override|params|private|protected|public|readonly|record|ref|return|sbyte|sealed|short|sizeof|stackalloc|static|string|struct|switch|this|throw|true|try|typeof|uint|ulong|unchecked|unsafe|ushort|using|var|virtual|void|volatile|while|yield)\b/,
        'number': /(?:\b0x[\da-f]+|(?:\b\d+(?:\.\d*)?|\B\.\d+)(?:e[+-]?\d+)?)[dflm]?/i
    });
    Prism.languages.cs = Prism.languages.csharp;

    /* Java */
    Prism.languages.java = Prism.languages.extend('clike', {
        'keyword': /\b(?:abstract|assert|boolean|break|byte|case|catch|char|class|const|continue|default|do|double|else|enum|exports|extends|final|finally|float|for|goto|if|implements|import|instanceof|int|interface|long|module|native|new|non-sealed|null|open|opens|package|permits|private|protected|provides|public|record|requires|return|sealed|short|static|strictfp|super|switch|synchronized|this|throw|throws|to|transient|transitive|try|uses|var|void|volatile|while|with|yield)\b/,
        'number': /\b0b[01][01_]*L?\b|\b0x(?:\.[\da-f_p+-]+|[\da-f_]+(?:\.[\da-f_]+)?(?:p[+-]?\d+)?)[dfl]?\b|(?:\b\d[\d_]*(?:\.[\d_]+)?|\B\.\d[\d_]+)(?:e[+-]?\d[\d_]+)?[dfl]?/i
    });

    /* C */
    Prism.languages.c = Prism.languages.extend('clike', {
        'keyword': /\b(?:_Alignas|_Alignof|_Atomic|_Bool|_Complex|_Generic|_Imaginary|_Noreturn|_Static_assert|_Thread_local|asm|auto|break|case|char|const|continue|default|do|double|else|enum|extern|float|for|goto|if|inline|int|long|register|restrict|return|short|signed|sizeof|static|struct|switch|typedef|union|unsigned|void|volatile|while)\b/,
        'number': /(?:\b0x(?:[\da-f]+(?:\.[\da-f]+)?|\.[\da-f]+)(?:p[+-]?\d+)?|(?:\b\d+(?:\.\d+)?|\B\.\d+)(?:e[+-]?\d+)?)[ful]{0,4}/i
    });

    /* C++ */
    Prism.languages.cpp = Prism.languages.extend('c', {
        'keyword': /\b(?:alignas|alignof|and|and_eq|asm|atomic_cancel|atomic_commit|atomic_noexcept|auto|bitand|bitor|bool|break|case|catch|char|char16_t|char32_t|char8_t|class|co_await|co_return|co_yield|compl|concept|const|const_cast|consteval|constexpr|constinit|continue|decltype|default|delete|do|double|dynamic_cast|else|enum|explicit|export|extern|false|final|float|for|friend|goto|if|import|inline|int|long|module|mutable|namespace|new|noexcept|not|not_eq|nullptr|operator|or|or_eq|override|private|protected|public|reflexpr|register|reinterpret_cast|requires|return|short|signed|sizeof|static|static_assert|static_cast|struct|switch|synchronized|template|this|thread_local|throw|true|try|typedef|typeid|typename|union|unsigned|using|virtual|void|volatile|wchar_t|while|xor|xor_eq)\b/
    });

    global.Prism = Prism;
})(typeof window !== 'undefined' ? window : globalThis);
