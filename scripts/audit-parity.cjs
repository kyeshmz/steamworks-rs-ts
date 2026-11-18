// Run from the repository root after generating client.d.ts.
const fs = require('node:fs')
const { execFileSync } = require('node:child_process')
const { createHash } = require('node:crypto')
const ts = require('typescript/unstable/ast')
const { API } = require('typescript/unstable/sync')
const { createVirtualFileSystem } = require('typescript/unstable/fs')
const dir = 'docs/coverage/'
const lock = JSON.parse(fs.readFileSync(dir + 'snapshot.json'))
const ffiLock = JSON.parse(fs.readFileSync(dir + 'parity-ffi-forks.json'))
const ffi = ffiLock.upstream
const git = (...args) => execFileSync('git', args, { encoding: 'utf8' }).trimEnd()
const blob = (sha, path) => git('show', `${sha}:${path}`)
const sources = createVirtualFileSystem({})
const compiler = new API({ fs: sources })
function ast(path, text) {
    const hash = createHash('sha256').update(text).digest('hex')
    const file = `${process.cwd().replaceAll('\\', '/')}/.audit/${hash}/${path}`
    sources.writeFile(file, text)
    const snapshot = compiler.updateSnapshot({ openFiles: [file] })
    const source = snapshot.getDefaultProjectForFile(file).program.getSourceFile(file)
    snapshot.dispose()
    compiler.updateSnapshot({ closeFiles: [file] }).dispose()
    return source
}
const isPublic = n => !(n.modifiers || []).some(m => [ts.SyntaxKind.PrivateKeyword, ts.SyntaxKind.ProtectedKeyword].includes(m.kind))
function symbols(file, scope = '', classScope) {
    const names = new Set()
    function walk(n, prefix) {
        if (!isPublic(n)) return
        if (ts.isModuleDeclaration(n) || ts.isClassDeclaration(n)) {
            const group = ts.isClassDeclaration(n) && classScope !== undefined ? classScope : prefix + n.name.text + '.'
            n.forEachChild(child => walk(child, group))
        } else if (ts.isVariableStatement(n) && n.modifiers?.some(m => m.kind === ts.SyntaxKind.ExportKeyword)) {
            for (const decl of n.declarationList.declarations) {
                if (decl.type && ts.isFunctionTypeNode(decl.type)) names.add(prefix + decl.name.text)
            }
        } else if (ts.isFunctionDeclaration(n) || ts.isMethodDeclaration(n) || ts.isGetAccessorDeclaration(n)) {
            names.add(prefix + n.name.text)
        } else if (ts.isEnumDeclaration(n) && prefix === 'callback.' && n.name.text === 'SteamCallback') {
            for (const member of n.members) names.add('callback.SteamCallback.' + member.name.text)
        } else if (ts.isSourceFile(n) || ts.isModuleBlock(n)) {
            n.forEachChild(child => walk(child, prefix))
        }
    }
    walk(file, scope)
    return names
}
const memo = new Map()
function declarations(sha) {
    const ids = git('ls-tree', sha, '--', 'client.d.ts', 'index.d.ts')
    if (memo.has(ids)) return memo.get(ids)
    const names = new Set()
    for (const line of ids.split('\n').filter(Boolean)) {
        const [meta, path] = line.split('\t')
        const text = git('cat-file', 'blob', meta.split(' ')[2])
        for (const name of symbols(ast(path, text))) names.add(name)
    }
    memo.set(ids, names)
    return names
}
const group = () => new Map()
const donors = group(), detached = group()
const callbacks = group()
const isCallback = name => name.startsWith('callback.SteamCallback.')
function add(map, name, source) {
    if (!map.has(name)) map.set(name, new Set())
    map.get(name).add(source)
}
const base = declarations(lock.upstream)
const tips = new Map()
for (const [repo,,sha] of lock.forks) { if (sha) add(tips, sha, repo) }
for (const [id,sha] of lock.pulls) add(tips, sha, '#' + id)
const unrelated = lock.forks.find(([repo]) => repo === 'Gurigraphics/steamworks.js')[2]
for (const [sha, sources] of tips) {
    if (sha === unrelated) continue
    const ancestor = git('merge-base', lock.upstream, sha)
    if (ancestor === sha) continue
    const old = declarations(ancestor)
    for (const name of declarations(sha)) {
        if (!base.has(name) && !old.has(name)) for (const source of sources) add(isCallback(name) ? callbacks : donors, name, source)
    }
}
for (const m of blob(unrelated, 'client/index.ts').matchAll(/export \* from '\.\/(\w+)\.js'/g)) {
    const path = `client/${m[1]}.ts`
    for (const name of symbols(ast(path, blob(unrelated, path)))) add(detached, name, path)
}
function ffiSurface(sha) {
    const api = group()
    const sdk = ast('src/steam.ts', blob(sha, 'src/steam.ts'))
    for (const name of symbols(sdk, '', 'sdk.')) add(api, name, 'src/steam.ts')
    for (const manager of sdk.statements.find(ts.isClassDeclaration).members) {
        if (!manager.modifiers?.some(m => m.kind === ts.SyntaxKind.ReadonlyKeyword) || !isPublic(manager)) continue
        const path = `src/internal/${manager.type.typeName.text}.ts`
        for (const name of symbols(ast(path, blob(sha, path)), '', manager.name.text + '.')) add(api, name, path)
    }
    for (const type of ['SteamOverlay', 'SteamLogger']) {
        const path = `src/internal/${type}.ts`
        for (const name of symbols(ast(path, blob(sha, path)))) add(api, name, path)
    }
    for (const match of blob(sha, 'src/types/index.ts').matchAll(/export \* from '\.\/(\w+)'/g)) {
        const path = `src/types/${match[1]}.ts`
        for (const n of ast(path, blob(sha, path)).statements) {
            if (ts.isFunctionDeclaration(n) && n.modifiers?.some(m => m.kind === ts.SyntaxKind.ExportKeyword)) {
                add(api, n.name.text, path)
            }
        }
    }
    return api
}
const independent = ffiSurface(ffi)
const ffiForkRows = ffiLock.forks.map(([repo, branch, tip]) => {
    const base = git('merge-base', ffi, tip)
    const changed = git('diff', '--name-only', base, tip)
    let extra = []
    if (changed) {
        const old = ffiSurface(base)
        extra = [...ffiSurface(tip).keys()].filter(name => !old.has(name) && !independent.has(name)).sort()
    }
    return `| [${repo}](https://github.com/${repo}/tree/${tip}) | ${branch} | \`${base}\` | ${changed.split('\n').filter(Boolean).join(', ') || 'none'} | ${extra.join(', ') || 'none'} |`
})
const inputs = ['client.d.ts', 'index.d.ts', 'index.js']
const loader = fs.readFileSync('index.js', 'utf8')
const bindings = new Map([...loader.matchAll(/^const (\w+) = nativeBinding\.(\w+);?$/gm)].map(m => [m[1], m[2]]))
const rootGroups = new Map()
for (const [, name, direct, variable] of loader.matchAll(/^module\.exports\.(\w+) = (?:nativeBinding\.(\w+)|(\w+));?$/gm)) {
    const native = direct || bindings.get(variable)
    if (native) rootGroups.set(native, name)
}
const local = new Set(inputs.filter(path => path.endsWith('.d.ts')).flatMap(path => {
    const names = symbols(ast(path, fs.readFileSync(path, 'utf8')))
    if (path === 'client.d.ts') for (const n of ['init', 'runCallbacks', 'restartAppIfNecessary']) names.delete(n)
    return [...names].map(name => {
        const [group, ...members] = name.split('.')
        return path === 'client.d.ts' && members.length && rootGroups.has(group) ? [rootGroups.get(group), ...members].join('.') : name
    })
}))
compiler.close()
const rules = JSON.parse(fs.readFileSync(dir + 'parity-rules.json', 'utf8'))
const seenRules = new Set()
function assess(name, pin) {
    const key = pin === ffi && rules['ffi:' + name] ? 'ffi:' + name : name
    if (rules[key]) {
        seenRules.add(key)
        const [status, value] = rules[key]
        if (status === 'alias' && !local.has(value)) throw Error(`Invalid target: ${name} -> ${value}`)
        if (status === 'gap' && local.has(name)) throw Error(`Recheck gap: ${name}`)
        return [status, value]
    }
    if (local.has(name)) return ['same name', name]
    const normalized = name.replace(/^(networkingSockets|sockets)\./, 'networking_sockets.').replace(/^leaderboard\./, 'leaderboards.').toLowerCase()
    const match = [...local].find(n => n.toLowerCase() === normalized)
    return match ? ['name variant', match] : ['review', '']
}
const result = ['# Public callable ledger', '', 'Generated by `node scripts/audit-parity.cjs`. Read [scope, priorities and reproduction](parity.md) before using these counts.', '',
    ...inputs.map(path => `- \`${path}\` SHA-256: \`${createHash('sha256').update(fs.readFileSync(path)).digest('hex')}\``),
    `- ${[...local].filter(n => !isCallback(n)).length} distinct local callable/accessor paths and ${[...local].filter(isCallback).length} callback identifiers. ${tips.size} distinct pinned fork/PR commit IDs.`, '']
for (const [title,map,pin] of [['Related fork and PR additions',donors], ['Related callback identifier additions',callbacks], ['Gurigraphics package entry',detached,unrelated], ['ArtyProf SDK, managers and other entry exports',independent,ffi]]) {
    const tally = {}
    const rows = [...map.keys()].sort().map(name => {
        const [status, value] = assess(name, pin)
        tally[status] = (tally[status] || 0) + 1
        return `| \`${name}\` | ${status} | ${value} | ${[...map.get(name)].join(', ')} |`
    })
    result.push(`## ${title}`, '', `${rows.length} distinct paths. ${Object.entries(tally).map(([k,v]) => `${k}: ${v}`).join('; ')}.`, '',
        pin ? `Commit: \`${pin}\`.` : 'Source identifiers resolve to exact commit IDs in [snapshot.json](snapshot.json).', '',
        '| Donor path | Finding | Local path or reviewed gap | Pinned source |', '|---|---|---|---|', ...rows, '')
}
for (const key of Object.keys(rules)) if (!seenRules.has(key)) throw Error(`Unused rule: ${key}`)
result.push('## ArtyProf fork default branches', '', 'Changed paths include implementation edits without new method names. See [the source review](parity.md#artyprof-fork-network) for their effect.', '',
    '| Fork at pinned tip | Branch | Merge base against pinned main | Changed paths since base | Additional callable paths absent from base and main |',
    '|---|---|---|---|---|', ...ffiForkRows, '')
const path = dir + 'parity-ledger.md'
const output = result.join('\n')
if (process.argv[2] === '--stdout') {
    process.stdout.write(output)
} else if (process.argv[2] === '--check') {
    if (fs.readFileSync(path, 'utf8') !== output) throw Error('Stale ' + path)
} else if (process.argv.length === 2) fs.writeFileSync(path, output)
else throw Error('Expected --check, --stdout or no arguments')
if (process.argv[2] !== '--stdout') console.log(result.filter(x => x.includes('distinct')).join('\n'))
