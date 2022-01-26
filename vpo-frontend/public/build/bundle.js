
(function(l, r) { if (!l || l.getElementById('livereloadscript')) return; r = l.createElement('script'); r.async = 1; r.src = '//' + (self.location.host || 'localhost').split(':')[0] + ':35729/livereload.js?snipver=1'; r.id = 'livereloadscript'; l.getElementsByTagName('head')[0].appendChild(r) })(self.document);
var app = (function () {
    'use strict';

    function noop() { }
    function assign(tar, src) {
        // @ts-ignore
        for (const k in src)
            tar[k] = src[k];
        return tar;
    }
    function add_location(element, file, line, column, char) {
        element.__svelte_meta = {
            loc: { file, line, column, char }
        };
    }
    function run(fn) {
        return fn();
    }
    function blank_object() {
        return Object.create(null);
    }
    function run_all(fns) {
        fns.forEach(run);
    }
    function is_function(thing) {
        return typeof thing === 'function';
    }
    function safe_not_equal(a, b) {
        return a != a ? b == b : a !== b || ((a && typeof a === 'object') || typeof a === 'function');
    }
    function is_empty(obj) {
        return Object.keys(obj).length === 0;
    }
    function append(target, node) {
        target.appendChild(node);
    }
    function insert(target, node, anchor) {
        target.insertBefore(node, anchor || null);
    }
    function detach(node) {
        node.parentNode.removeChild(node);
    }
    function destroy_each(iterations, detaching) {
        for (let i = 0; i < iterations.length; i += 1) {
            if (iterations[i])
                iterations[i].d(detaching);
        }
    }
    function element(name) {
        return document.createElement(name);
    }
    function svg_element(name) {
        return document.createElementNS('http://www.w3.org/2000/svg', name);
    }
    function text(data) {
        return document.createTextNode(data);
    }
    function space() {
        return text(' ');
    }
    function empty() {
        return text('');
    }
    function listen(node, event, handler, options) {
        node.addEventListener(event, handler, options);
        return () => node.removeEventListener(event, handler, options);
    }
    function attr(node, attribute, value) {
        if (value == null)
            node.removeAttribute(attribute);
        else if (node.getAttribute(attribute) !== value)
            node.setAttribute(attribute, value);
    }
    function to_number(value) {
        return value === '' ? null : +value;
    }
    function children(element) {
        return Array.from(element.childNodes);
    }
    function set_input_value(input, value) {
        input.value = value == null ? '' : value;
    }
    function set_style(node, key, value, important) {
        node.style.setProperty(key, value, important ? 'important' : '');
    }
    function select_option(select, value) {
        for (let i = 0; i < select.options.length; i += 1) {
            const option = select.options[i];
            if (option.__value === value) {
                option.selected = true;
                return;
            }
        }
        select.selectedIndex = -1; // no option should be selected
    }
    function select_value(select) {
        const selected_option = select.querySelector(':checked') || select.options[0];
        return selected_option && selected_option.__value;
    }
    function toggle_class(element, name, toggle) {
        element.classList[toggle ? 'add' : 'remove'](name);
    }
    function custom_event(type, detail, bubbles = false) {
        const e = document.createEvent('CustomEvent');
        e.initCustomEvent(type, bubbles, false, detail);
        return e;
    }

    let current_component;
    function set_current_component(component) {
        current_component = component;
    }
    function get_current_component() {
        if (!current_component)
            throw new Error('Function called outside component initialization');
        return current_component;
    }
    function onMount(fn) {
        get_current_component().$$.on_mount.push(fn);
    }

    const dirty_components = [];
    const binding_callbacks = [];
    const render_callbacks = [];
    const flush_callbacks = [];
    const resolved_promise = Promise.resolve();
    let update_scheduled = false;
    function schedule_update() {
        if (!update_scheduled) {
            update_scheduled = true;
            resolved_promise.then(flush);
        }
    }
    function add_render_callback(fn) {
        render_callbacks.push(fn);
    }
    // flush() calls callbacks in this order:
    // 1. All beforeUpdate callbacks, in order: parents before children
    // 2. All bind:this callbacks, in reverse order: children before parents.
    // 3. All afterUpdate callbacks, in order: parents before children. EXCEPT
    //    for afterUpdates called during the initial onMount, which are called in
    //    reverse order: children before parents.
    // Since callbacks might update component values, which could trigger another
    // call to flush(), the following steps guard against this:
    // 1. During beforeUpdate, any updated components will be added to the
    //    dirty_components array and will cause a reentrant call to flush(). Because
    //    the flush index is kept outside the function, the reentrant call will pick
    //    up where the earlier call left off and go through all dirty components. The
    //    current_component value is saved and restored so that the reentrant call will
    //    not interfere with the "parent" flush() call.
    // 2. bind:this callbacks cannot trigger new flush() calls.
    // 3. During afterUpdate, any updated components will NOT have their afterUpdate
    //    callback called a second time; the seen_callbacks set, outside the flush()
    //    function, guarantees this behavior.
    const seen_callbacks = new Set();
    let flushidx = 0; // Do *not* move this inside the flush() function
    function flush() {
        const saved_component = current_component;
        do {
            // first, call beforeUpdate functions
            // and update components
            while (flushidx < dirty_components.length) {
                const component = dirty_components[flushidx];
                flushidx++;
                set_current_component(component);
                update(component.$$);
            }
            set_current_component(null);
            dirty_components.length = 0;
            flushidx = 0;
            while (binding_callbacks.length)
                binding_callbacks.pop()();
            // then, once components are updated, call
            // afterUpdate functions. This may cause
            // subsequent updates...
            for (let i = 0; i < render_callbacks.length; i += 1) {
                const callback = render_callbacks[i];
                if (!seen_callbacks.has(callback)) {
                    // ...so guard against infinite loops
                    seen_callbacks.add(callback);
                    callback();
                }
            }
            render_callbacks.length = 0;
        } while (dirty_components.length);
        while (flush_callbacks.length) {
            flush_callbacks.pop()();
        }
        update_scheduled = false;
        seen_callbacks.clear();
        set_current_component(saved_component);
    }
    function update($$) {
        if ($$.fragment !== null) {
            $$.update();
            run_all($$.before_update);
            const dirty = $$.dirty;
            $$.dirty = [-1];
            $$.fragment && $$.fragment.p($$.ctx, dirty);
            $$.after_update.forEach(add_render_callback);
        }
    }
    const outroing = new Set();
    let outros;
    function group_outros() {
        outros = {
            r: 0,
            c: [],
            p: outros // parent group
        };
    }
    function check_outros() {
        if (!outros.r) {
            run_all(outros.c);
        }
        outros = outros.p;
    }
    function transition_in(block, local) {
        if (block && block.i) {
            outroing.delete(block);
            block.i(local);
        }
    }
    function transition_out(block, local, detach, callback) {
        if (block && block.o) {
            if (outroing.has(block))
                return;
            outroing.add(block);
            outros.c.push(() => {
                outroing.delete(block);
                if (callback) {
                    if (detach)
                        block.d(1);
                    callback();
                }
            });
            block.o(local);
        }
    }

    const globals = (typeof window !== 'undefined'
        ? window
        : typeof globalThis !== 'undefined'
            ? globalThis
            : global);
    function outro_and_destroy_block(block, lookup) {
        transition_out(block, 1, 1, () => {
            lookup.delete(block.key);
        });
    }
    function update_keyed_each(old_blocks, dirty, get_key, dynamic, ctx, list, lookup, node, destroy, create_each_block, next, get_context) {
        let o = old_blocks.length;
        let n = list.length;
        let i = o;
        const old_indexes = {};
        while (i--)
            old_indexes[old_blocks[i].key] = i;
        const new_blocks = [];
        const new_lookup = new Map();
        const deltas = new Map();
        i = n;
        while (i--) {
            const child_ctx = get_context(ctx, list, i);
            const key = get_key(child_ctx);
            let block = lookup.get(key);
            if (!block) {
                block = create_each_block(key, child_ctx);
                block.c();
            }
            else if (dynamic) {
                block.p(child_ctx, dirty);
            }
            new_lookup.set(key, new_blocks[i] = block);
            if (key in old_indexes)
                deltas.set(key, Math.abs(i - old_indexes[key]));
        }
        const will_move = new Set();
        const did_move = new Set();
        function insert(block) {
            transition_in(block, 1);
            block.m(node, next);
            lookup.set(block.key, block);
            next = block.first;
            n--;
        }
        while (o && n) {
            const new_block = new_blocks[n - 1];
            const old_block = old_blocks[o - 1];
            const new_key = new_block.key;
            const old_key = old_block.key;
            if (new_block === old_block) {
                // do nothing
                next = new_block.first;
                o--;
                n--;
            }
            else if (!new_lookup.has(old_key)) {
                // remove old block
                destroy(old_block, lookup);
                o--;
            }
            else if (!lookup.has(new_key) || will_move.has(new_key)) {
                insert(new_block);
            }
            else if (did_move.has(old_key)) {
                o--;
            }
            else if (deltas.get(new_key) > deltas.get(old_key)) {
                did_move.add(new_key);
                insert(new_block);
            }
            else {
                will_move.add(old_key);
                o--;
            }
        }
        while (o--) {
            const old_block = old_blocks[o];
            if (!new_lookup.has(old_block.key))
                destroy(old_block, lookup);
        }
        while (n)
            insert(new_blocks[n - 1]);
        return new_blocks;
    }
    function validate_each_keys(ctx, list, get_context, get_key) {
        const keys = new Set();
        for (let i = 0; i < list.length; i++) {
            const key = get_key(get_context(ctx, list, i));
            if (keys.has(key)) {
                throw new Error('Cannot have duplicate keys in a keyed each');
            }
            keys.add(key);
        }
    }

    function get_spread_update(levels, updates) {
        const update = {};
        const to_null_out = {};
        const accounted_for = { $$scope: 1 };
        let i = levels.length;
        while (i--) {
            const o = levels[i];
            const n = updates[i];
            if (n) {
                for (const key in o) {
                    if (!(key in n))
                        to_null_out[key] = 1;
                }
                for (const key in n) {
                    if (!accounted_for[key]) {
                        update[key] = n[key];
                        accounted_for[key] = 1;
                    }
                }
                levels[i] = n;
            }
            else {
                for (const key in o) {
                    accounted_for[key] = 1;
                }
            }
        }
        for (const key in to_null_out) {
            if (!(key in update))
                update[key] = undefined;
        }
        return update;
    }
    function get_spread_object(spread_props) {
        return typeof spread_props === 'object' && spread_props !== null ? spread_props : {};
    }
    function create_component(block) {
        block && block.c();
    }
    function mount_component(component, target, anchor, customElement) {
        const { fragment, on_mount, on_destroy, after_update } = component.$$;
        fragment && fragment.m(target, anchor);
        if (!customElement) {
            // onMount happens before the initial afterUpdate
            add_render_callback(() => {
                const new_on_destroy = on_mount.map(run).filter(is_function);
                if (on_destroy) {
                    on_destroy.push(...new_on_destroy);
                }
                else {
                    // Edge case - component was destroyed immediately,
                    // most likely as a result of a binding initialising
                    run_all(new_on_destroy);
                }
                component.$$.on_mount = [];
            });
        }
        after_update.forEach(add_render_callback);
    }
    function destroy_component(component, detaching) {
        const $$ = component.$$;
        if ($$.fragment !== null) {
            run_all($$.on_destroy);
            $$.fragment && $$.fragment.d(detaching);
            // TODO null out other refs, including component.$$ (but need to
            // preserve final state?)
            $$.on_destroy = $$.fragment = null;
            $$.ctx = [];
        }
    }
    function make_dirty(component, i) {
        if (component.$$.dirty[0] === -1) {
            dirty_components.push(component);
            schedule_update();
            component.$$.dirty.fill(0);
        }
        component.$$.dirty[(i / 31) | 0] |= (1 << (i % 31));
    }
    function init(component, options, instance, create_fragment, not_equal, props, append_styles, dirty = [-1]) {
        const parent_component = current_component;
        set_current_component(component);
        const $$ = component.$$ = {
            fragment: null,
            ctx: null,
            // state
            props,
            update: noop,
            not_equal,
            bound: blank_object(),
            // lifecycle
            on_mount: [],
            on_destroy: [],
            on_disconnect: [],
            before_update: [],
            after_update: [],
            context: new Map(options.context || (parent_component ? parent_component.$$.context : [])),
            // everything else
            callbacks: blank_object(),
            dirty,
            skip_bound: false,
            root: options.target || parent_component.$$.root
        };
        append_styles && append_styles($$.root);
        let ready = false;
        $$.ctx = instance
            ? instance(component, options.props || {}, (i, ret, ...rest) => {
                const value = rest.length ? rest[0] : ret;
                if ($$.ctx && not_equal($$.ctx[i], $$.ctx[i] = value)) {
                    if (!$$.skip_bound && $$.bound[i])
                        $$.bound[i](value);
                    if (ready)
                        make_dirty(component, i);
                }
                return ret;
            })
            : [];
        $$.update();
        ready = true;
        run_all($$.before_update);
        // `false` as a special case of no DOM component
        $$.fragment = create_fragment ? create_fragment($$.ctx) : false;
        if (options.target) {
            if (options.hydrate) {
                const nodes = children(options.target);
                // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
                $$.fragment && $$.fragment.l(nodes);
                nodes.forEach(detach);
            }
            else {
                // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
                $$.fragment && $$.fragment.c();
            }
            if (options.intro)
                transition_in(component.$$.fragment);
            mount_component(component, options.target, options.anchor, options.customElement);
            flush();
        }
        set_current_component(parent_component);
    }
    /**
     * Base class for Svelte components. Used when dev=false.
     */
    class SvelteComponent {
        $destroy() {
            destroy_component(this, 1);
            this.$destroy = noop;
        }
        $on(type, callback) {
            const callbacks = (this.$$.callbacks[type] || (this.$$.callbacks[type] = []));
            callbacks.push(callback);
            return () => {
                const index = callbacks.indexOf(callback);
                if (index !== -1)
                    callbacks.splice(index, 1);
            };
        }
        $set($$props) {
            if (this.$$set && !is_empty($$props)) {
                this.$$.skip_bound = true;
                this.$$set($$props);
                this.$$.skip_bound = false;
            }
        }
    }

    function dispatch_dev(type, detail) {
        document.dispatchEvent(custom_event(type, Object.assign({ version: '3.44.3' }, detail), true));
    }
    function append_dev(target, node) {
        dispatch_dev('SvelteDOMInsert', { target, node });
        append(target, node);
    }
    function insert_dev(target, node, anchor) {
        dispatch_dev('SvelteDOMInsert', { target, node, anchor });
        insert(target, node, anchor);
    }
    function detach_dev(node) {
        dispatch_dev('SvelteDOMRemove', { node });
        detach(node);
    }
    function listen_dev(node, event, handler, options, has_prevent_default, has_stop_propagation) {
        const modifiers = options === true ? ['capture'] : options ? Array.from(Object.keys(options)) : [];
        if (has_prevent_default)
            modifiers.push('preventDefault');
        if (has_stop_propagation)
            modifiers.push('stopPropagation');
        dispatch_dev('SvelteDOMAddEventListener', { node, event, handler, modifiers });
        const dispose = listen(node, event, handler, options);
        return () => {
            dispatch_dev('SvelteDOMRemoveEventListener', { node, event, handler, modifiers });
            dispose();
        };
    }
    function attr_dev(node, attribute, value) {
        attr(node, attribute, value);
        if (value == null)
            dispatch_dev('SvelteDOMRemoveAttribute', { node, attribute });
        else
            dispatch_dev('SvelteDOMSetAttribute', { node, attribute, value });
    }
    function prop_dev(node, property, value) {
        node[property] = value;
        dispatch_dev('SvelteDOMSetProperty', { node, property, value });
    }
    function set_data_dev(text, data) {
        data = '' + data;
        if (text.wholeText === data)
            return;
        dispatch_dev('SvelteDOMSetData', { node: text, data });
        text.data = data;
    }
    function validate_each_argument(arg) {
        if (typeof arg !== 'string' && !(arg && typeof arg === 'object' && 'length' in arg)) {
            let msg = '{#each} only iterates over array-like objects.';
            if (typeof Symbol === 'function' && arg && Symbol.iterator in arg) {
                msg += ' You can use a spread to convert this iterable into an array.';
            }
            throw new Error(msg);
        }
    }
    function validate_slots(name, slot, keys) {
        for (const slot_key of Object.keys(slot)) {
            if (!~keys.indexOf(slot_key)) {
                console.warn(`<${name}> received an unexpected slot "${slot_key}".`);
            }
        }
    }
    /**
     * Base class for Svelte components with some minor dev-enhancements. Used when dev=true.
     */
    class SvelteComponentDev extends SvelteComponent {
        constructor(options) {
            if (!options || (!options.target && !options.$$inline)) {
                throw new Error("'target' is a required option");
            }
            super();
        }
        $destroy() {
            super.$destroy();
            this.$destroy = () => {
                console.warn('Component was already destroyed'); // eslint-disable-line no-console
            };
        }
        $capture_state() { }
        $inject_state() { }
    }

    /* src/node-editor/NumberProperty.svelte generated by Svelte v3.44.3 */

    const file$8 = "src/node-editor/NumberProperty.svelte";

    function create_fragment$8(ctx) {
    	let input;
    	let mounted;
    	let dispose;

    	const block = {
    		c: function create() {
    			input = element("input");
    			attr_dev(input, "type", "number");
    			set_style(input, "width", "100%");
    			attr_dev(input, "class", "svelte-f9449i");
    			add_location(input, file$8, 17, 0, 236);
    		},
    		l: function claim(nodes) {
    			throw new Error("options.hydrate only works if the component was compiled with the `hydratable: true` option");
    		},
    		m: function mount(target, anchor) {
    			insert_dev(target, input, anchor);
    			set_input_value(input, /*value*/ ctx[0]);

    			if (!mounted) {
    				dispose = listen_dev(input, "input", /*input_input_handler*/ ctx[3]);
    				mounted = true;
    			}
    		},
    		p: function update(ctx, [dirty]) {
    			if (dirty & /*value*/ 1 && to_number(input.value) !== /*value*/ ctx[0]) {
    				set_input_value(input, /*value*/ ctx[0]);
    			}
    		},
    		i: noop,
    		o: noop,
    		d: function destroy(detaching) {
    			if (detaching) detach_dev(input);
    			mounted = false;
    			dispose();
    		}
    	};

    	dispatch_dev("SvelteRegisterBlock", {
    		block,
    		id: create_fragment$8.name,
    		type: "component",
    		source: "",
    		ctx
    	});

    	return block;
    }

    function instance$8($$self, $$props, $$invalidate) {
    	let { $$slots: slots = {}, $$scope } = $$props;
    	validate_slots('NumberProperty', slots, []);

    	let { onchange = function () {
    		
    	} } = $$props;

    	let { value } = $$props;
    	let firstTime = true;
    	const writable_props = ['onchange', 'value'];

    	Object.keys($$props).forEach(key => {
    		if (!~writable_props.indexOf(key) && key.slice(0, 2) !== '$$' && key !== 'slot') console.warn(`<NumberProperty> was created with unknown prop '${key}'`);
    	});

    	function input_input_handler() {
    		value = to_number(this.value);
    		$$invalidate(0, value);
    	}

    	$$self.$$set = $$props => {
    		if ('onchange' in $$props) $$invalidate(1, onchange = $$props.onchange);
    		if ('value' in $$props) $$invalidate(0, value = $$props.value);
    	};

    	$$self.$capture_state = () => ({ onchange, value, firstTime });

    	$$self.$inject_state = $$props => {
    		if ('onchange' in $$props) $$invalidate(1, onchange = $$props.onchange);
    		if ('value' in $$props) $$invalidate(0, value = $$props.value);
    		if ('firstTime' in $$props) $$invalidate(2, firstTime = $$props.firstTime);
    	};

    	if ($$props && "$$inject" in $$props) {
    		$$self.$inject_state($$props.$$inject);
    	}

    	$$self.$$.update = () => {
    		if ($$self.$$.dirty & /*firstTime, value, onchange*/ 7) {
    			{
    				if (!firstTime && value !== undefined) {
    					onchange(value);
    				}

    				if (firstTime) {
    					$$invalidate(2, firstTime = false);
    				}
    			}
    		}
    	};

    	return [value, onchange, firstTime, input_input_handler];
    }

    class NumberProperty extends SvelteComponentDev {
    	constructor(options) {
    		super(options);
    		init(this, options, instance$8, create_fragment$8, safe_not_equal, { onchange: 1, value: 0 });

    		dispatch_dev("SvelteRegisterComponent", {
    			component: this,
    			tagName: "NumberProperty",
    			options,
    			id: create_fragment$8.name
    		});

    		const { ctx } = this.$$;
    		const props = options.props || {};

    		if (/*value*/ ctx[0] === undefined && !('value' in props)) {
    			console.warn("<NumberProperty> was created without expected prop 'value'");
    		}
    	}

    	get onchange() {
    		throw new Error("<NumberProperty>: Props cannot be read directly from the component instance unless compiling with 'accessors: true' or '<svelte:options accessors/>'");
    	}

    	set onchange(value) {
    		throw new Error("<NumberProperty>: Props cannot be set directly on the component instance unless compiling with 'accessors: true' or '<svelte:options accessors/>'");
    	}

    	get value() {
    		throw new Error("<NumberProperty>: Props cannot be read directly from the component instance unless compiling with 'accessors: true' or '<svelte:options accessors/>'");
    	}

    	set value(value) {
    		throw new Error("<NumberProperty>: Props cannot be set directly on the component instance unless compiling with 'accessors: true' or '<svelte:options accessors/>'");
    	}
    }

    /* src/node-editor/OptionsProperty.svelte generated by Svelte v3.44.3 */

    const { Object: Object_1$1 } = globals;
    const file$7 = "src/node-editor/OptionsProperty.svelte";

    function get_each_context$1(ctx, list, i) {
    	const child_ctx = ctx.slice();
    	child_ctx[5] = list[i][0];
    	child_ctx[6] = list[i][1];
    	return child_ctx;
    }

    // (20:0) {#each Object.entries(states) as [i, state]}
    function create_each_block$1(ctx) {
    	let option;
    	let t_value = /*state*/ ctx[6] + "";
    	let t;
    	let option_value_value;

    	const block = {
    		c: function create() {
    			option = element("option");
    			t = text(t_value);
    			option.__value = option_value_value = /*i*/ ctx[5];
    			option.value = option.__value;
    			add_location(option, file$7, 20, 4, 351);
    		},
    		m: function mount(target, anchor) {
    			insert_dev(target, option, anchor);
    			append_dev(option, t);
    		},
    		p: function update(ctx, dirty) {
    			if (dirty & /*states*/ 2 && t_value !== (t_value = /*state*/ ctx[6] + "")) set_data_dev(t, t_value);

    			if (dirty & /*states*/ 2 && option_value_value !== (option_value_value = /*i*/ ctx[5])) {
    				prop_dev(option, "__value", option_value_value);
    				option.value = option.__value;
    			}
    		},
    		d: function destroy(detaching) {
    			if (detaching) detach_dev(option);
    		}
    	};

    	dispatch_dev("SvelteRegisterBlock", {
    		block,
    		id: create_each_block$1.name,
    		type: "each",
    		source: "(20:0) {#each Object.entries(states) as [i, state]}",
    		ctx
    	});

    	return block;
    }

    function create_fragment$7(ctx) {
    	let select;
    	let mounted;
    	let dispose;
    	let each_value = Object.entries(/*states*/ ctx[1]);
    	validate_each_argument(each_value);
    	let each_blocks = [];

    	for (let i = 0; i < each_value.length; i += 1) {
    		each_blocks[i] = create_each_block$1(get_each_context$1(ctx, each_value, i));
    	}

    	const block = {
    		c: function create() {
    			select = element("select");

    			for (let i = 0; i < each_blocks.length; i += 1) {
    				each_blocks[i].c();
    			}

    			attr_dev(select, "class", "svelte-vzlxul");
    			if (/*value*/ ctx[0] === void 0) add_render_callback(() => /*select_change_handler*/ ctx[4].call(select));
    			add_location(select, file$7, 18, 0, 274);
    		},
    		l: function claim(nodes) {
    			throw new Error("options.hydrate only works if the component was compiled with the `hydratable: true` option");
    		},
    		m: function mount(target, anchor) {
    			insert_dev(target, select, anchor);

    			for (let i = 0; i < each_blocks.length; i += 1) {
    				each_blocks[i].m(select, null);
    			}

    			select_option(select, /*value*/ ctx[0]);

    			if (!mounted) {
    				dispose = listen_dev(select, "change", /*select_change_handler*/ ctx[4]);
    				mounted = true;
    			}
    		},
    		p: function update(ctx, [dirty]) {
    			if (dirty & /*Object, states*/ 2) {
    				each_value = Object.entries(/*states*/ ctx[1]);
    				validate_each_argument(each_value);
    				let i;

    				for (i = 0; i < each_value.length; i += 1) {
    					const child_ctx = get_each_context$1(ctx, each_value, i);

    					if (each_blocks[i]) {
    						each_blocks[i].p(child_ctx, dirty);
    					} else {
    						each_blocks[i] = create_each_block$1(child_ctx);
    						each_blocks[i].c();
    						each_blocks[i].m(select, null);
    					}
    				}

    				for (; i < each_blocks.length; i += 1) {
    					each_blocks[i].d(1);
    				}

    				each_blocks.length = each_value.length;
    			}

    			if (dirty & /*value, Object, states*/ 3) {
    				select_option(select, /*value*/ ctx[0]);
    			}
    		},
    		i: noop,
    		o: noop,
    		d: function destroy(detaching) {
    			if (detaching) detach_dev(select);
    			destroy_each(each_blocks, detaching);
    			mounted = false;
    			dispose();
    		}
    	};

    	dispatch_dev("SvelteRegisterBlock", {
    		block,
    		id: create_fragment$7.name,
    		type: "component",
    		source: "",
    		ctx
    	});

    	return block;
    }

    function instance$7($$self, $$props, $$invalidate) {
    	let { $$slots: slots = {}, $$scope } = $$props;
    	validate_slots('OptionsProperty', slots, []);

    	let { onchange = function () {
    		
    	} } = $$props;

    	let { value } = $$props;
    	let { states } = $$props;
    	let firstTime = true;
    	const writable_props = ['onchange', 'value', 'states'];

    	Object_1$1.keys($$props).forEach(key => {
    		if (!~writable_props.indexOf(key) && key.slice(0, 2) !== '$$' && key !== 'slot') console.warn(`<OptionsProperty> was created with unknown prop '${key}'`);
    	});

    	function select_change_handler() {
    		value = select_value(this);
    		$$invalidate(0, value);
    		$$invalidate(1, states);
    	}

    	$$self.$$set = $$props => {
    		if ('onchange' in $$props) $$invalidate(2, onchange = $$props.onchange);
    		if ('value' in $$props) $$invalidate(0, value = $$props.value);
    		if ('states' in $$props) $$invalidate(1, states = $$props.states);
    	};

    	$$self.$capture_state = () => ({ onchange, value, states, firstTime });

    	$$self.$inject_state = $$props => {
    		if ('onchange' in $$props) $$invalidate(2, onchange = $$props.onchange);
    		if ('value' in $$props) $$invalidate(0, value = $$props.value);
    		if ('states' in $$props) $$invalidate(1, states = $$props.states);
    		if ('firstTime' in $$props) $$invalidate(3, firstTime = $$props.firstTime);
    	};

    	if ($$props && "$$inject" in $$props) {
    		$$self.$inject_state($$props.$$inject);
    	}

    	$$self.$$.update = () => {
    		if ($$self.$$.dirty & /*firstTime, value, onchange*/ 13) {
    			{
    				if (!firstTime && value !== undefined) {
    					onchange(value);
    				}

    				if (firstTime) {
    					$$invalidate(3, firstTime = false);
    				}
    			}
    		}
    	};

    	return [value, states, onchange, firstTime, select_change_handler];
    }

    class OptionsProperty extends SvelteComponentDev {
    	constructor(options) {
    		super(options);
    		init(this, options, instance$7, create_fragment$7, safe_not_equal, { onchange: 2, value: 0, states: 1 });

    		dispatch_dev("SvelteRegisterComponent", {
    			component: this,
    			tagName: "OptionsProperty",
    			options,
    			id: create_fragment$7.name
    		});

    		const { ctx } = this.$$;
    		const props = options.props || {};

    		if (/*value*/ ctx[0] === undefined && !('value' in props)) {
    			console.warn("<OptionsProperty> was created without expected prop 'value'");
    		}

    		if (/*states*/ ctx[1] === undefined && !('states' in props)) {
    			console.warn("<OptionsProperty> was created without expected prop 'states'");
    		}
    	}

    	get onchange() {
    		throw new Error("<OptionsProperty>: Props cannot be read directly from the component instance unless compiling with 'accessors: true' or '<svelte:options accessors/>'");
    	}

    	set onchange(value) {
    		throw new Error("<OptionsProperty>: Props cannot be set directly on the component instance unless compiling with 'accessors: true' or '<svelte:options accessors/>'");
    	}

    	get value() {
    		throw new Error("<OptionsProperty>: Props cannot be read directly from the component instance unless compiling with 'accessors: true' or '<svelte:options accessors/>'");
    	}

    	set value(value) {
    		throw new Error("<OptionsProperty>: Props cannot be set directly on the component instance unless compiling with 'accessors: true' or '<svelte:options accessors/>'");
    	}

    	get states() {
    		throw new Error("<OptionsProperty>: Props cannot be read directly from the component instance unless compiling with 'accessors: true' or '<svelte:options accessors/>'");
    	}

    	set states(value) {
    		throw new Error("<OptionsProperty>: Props cannot be set directly on the component instance unless compiling with 'accessors: true' or '<svelte:options accessors/>'");
    	}
    }

    /* src/node-editor/PropertyEditor.svelte generated by Svelte v3.44.3 */
    const file$6 = "src/node-editor/PropertyEditor.svelte";

    function create_fragment$6(ctx) {
    	let div9;
    	let div8;
    	let div3;
    	let div0;
    	let t1;
    	let div1;
    	let t2;
    	let div2;
    	let numberproperty;
    	let t3;
    	let div7;
    	let div4;
    	let t5;
    	let div5;
    	let t6;
    	let div6;
    	let optionsproperty;
    	let current;
    	numberproperty = new NumberProperty({ props: { value: 0 }, $$inline: true });

    	optionsproperty = new OptionsProperty({
    			props: {
    				states: { foo: "bar", baz: "qux" },
    				value: "baz"
    			},
    			$$inline: true
    		});

    	const block = {
    		c: function create() {
    			div9 = element("div");
    			div8 = element("div");
    			div3 = element("div");
    			div0 = element("div");
    			div0.textContent = "Number";
    			t1 = space();
    			div1 = element("div");
    			t2 = space();
    			div2 = element("div");
    			create_component(numberproperty.$$.fragment);
    			t3 = space();
    			div7 = element("div");
    			div4 = element("div");
    			div4.textContent = "Stuff";
    			t5 = space();
    			div5 = element("div");
    			t6 = space();
    			div6 = element("div");
    			create_component(optionsproperty.$$.fragment);
    			attr_dev(div0, "class", "prop-name svelte-90k3tc");
    			add_location(div0, file$6, 11, 12, 285);
    			attr_dev(div1, "class", "dividing-bar svelte-90k3tc");
    			add_location(div1, file$6, 12, 12, 333);
    			attr_dev(div2, "class", "prop-value svelte-90k3tc");
    			add_location(div2, file$6, 13, 12, 378);
    			attr_dev(div3, "class", "row svelte-90k3tc");
    			add_location(div3, file$6, 10, 8, 255);
    			attr_dev(div4, "class", "prop-name svelte-90k3tc");
    			add_location(div4, file$6, 16, 12, 490);
    			attr_dev(div5, "class", "dividing-bar svelte-90k3tc");
    			add_location(div5, file$6, 17, 12, 537);
    			attr_dev(div6, "class", "prop-value svelte-90k3tc");
    			add_location(div6, file$6, 18, 12, 582);
    			attr_dev(div7, "class", "row svelte-90k3tc");
    			add_location(div7, file$6, 15, 8, 460);
    			attr_dev(div8, "class", "container svelte-90k3tc");
    			add_location(div8, file$6, 9, 4, 223);
    			set_style(div9, "width", /*width*/ ctx[0] + "px");
    			set_style(div9, "height", /*height*/ ctx[1] + "px");
    			add_location(div9, file$6, 8, 0, 168);
    		},
    		l: function claim(nodes) {
    			throw new Error("options.hydrate only works if the component was compiled with the `hydratable: true` option");
    		},
    		m: function mount(target, anchor) {
    			insert_dev(target, div9, anchor);
    			append_dev(div9, div8);
    			append_dev(div8, div3);
    			append_dev(div3, div0);
    			append_dev(div3, t1);
    			append_dev(div3, div1);
    			append_dev(div3, t2);
    			append_dev(div3, div2);
    			mount_component(numberproperty, div2, null);
    			append_dev(div8, t3);
    			append_dev(div8, div7);
    			append_dev(div7, div4);
    			append_dev(div7, t5);
    			append_dev(div7, div5);
    			append_dev(div7, t6);
    			append_dev(div7, div6);
    			mount_component(optionsproperty, div6, null);
    			current = true;
    		},
    		p: function update(ctx, [dirty]) {
    			if (!current || dirty & /*width*/ 1) {
    				set_style(div9, "width", /*width*/ ctx[0] + "px");
    			}

    			if (!current || dirty & /*height*/ 2) {
    				set_style(div9, "height", /*height*/ ctx[1] + "px");
    			}
    		},
    		i: function intro(local) {
    			if (current) return;
    			transition_in(numberproperty.$$.fragment, local);
    			transition_in(optionsproperty.$$.fragment, local);
    			current = true;
    		},
    		o: function outro(local) {
    			transition_out(numberproperty.$$.fragment, local);
    			transition_out(optionsproperty.$$.fragment, local);
    			current = false;
    		},
    		d: function destroy(detaching) {
    			if (detaching) detach_dev(div9);
    			destroy_component(numberproperty);
    			destroy_component(optionsproperty);
    		}
    	};

    	dispatch_dev("SvelteRegisterBlock", {
    		block,
    		id: create_fragment$6.name,
    		type: "component",
    		source: "",
    		ctx
    	});

    	return block;
    }

    function instance$6($$self, $$props, $$invalidate) {
    	let { $$slots: slots = {}, $$scope } = $$props;
    	validate_slots('PropertyEditor', slots, []);
    	let { width } = $$props;
    	let { height } = $$props;
    	const writable_props = ['width', 'height'];

    	Object.keys($$props).forEach(key => {
    		if (!~writable_props.indexOf(key) && key.slice(0, 2) !== '$$' && key !== 'slot') console.warn(`<PropertyEditor> was created with unknown prop '${key}'`);
    	});

    	$$self.$$set = $$props => {
    		if ('width' in $$props) $$invalidate(0, width = $$props.width);
    		if ('height' in $$props) $$invalidate(1, height = $$props.height);
    	};

    	$$self.$capture_state = () => ({
    		NumberProperty,
    		OptionsProperty,
    		width,
    		height
    	});

    	$$self.$inject_state = $$props => {
    		if ('width' in $$props) $$invalidate(0, width = $$props.width);
    		if ('height' in $$props) $$invalidate(1, height = $$props.height);
    	};

    	if ($$props && "$$inject" in $$props) {
    		$$self.$inject_state($$props.$$inject);
    	}

    	return [width, height];
    }

    class PropertyEditor extends SvelteComponentDev {
    	constructor(options) {
    		super(options);
    		init(this, options, instance$6, create_fragment$6, safe_not_equal, { width: 0, height: 1 });

    		dispatch_dev("SvelteRegisterComponent", {
    			component: this,
    			tagName: "PropertyEditor",
    			options,
    			id: create_fragment$6.name
    		});

    		const { ctx } = this.$$;
    		const props = options.props || {};

    		if (/*width*/ ctx[0] === undefined && !('width' in props)) {
    			console.warn("<PropertyEditor> was created without expected prop 'width'");
    		}

    		if (/*height*/ ctx[1] === undefined && !('height' in props)) {
    			console.warn("<PropertyEditor> was created without expected prop 'height'");
    		}
    	}

    	get width() {
    		throw new Error("<PropertyEditor>: Props cannot be read directly from the component instance unless compiling with 'accessors: true' or '<svelte:options accessors/>'");
    	}

    	set width(value) {
    		throw new Error("<PropertyEditor>: Props cannot be set directly on the component instance unless compiling with 'accessors: true' or '<svelte:options accessors/>'");
    	}

    	get height() {
    		throw new Error("<PropertyEditor>: Props cannot be read directly from the component instance unless compiling with 'accessors: true' or '<svelte:options accessors/>'");
    	}

    	set height(value) {
    		throw new Error("<PropertyEditor>: Props cannot be set directly on the component instance unless compiling with 'accessors: true' or '<svelte:options accessors/>'");
    	}
    }

    /* src/node-editor/Socket.svelte generated by Svelte v3.44.3 */

    const { console: console_1$1 } = globals;
    const file$5 = "src/node-editor/Socket.svelte";

    // (15:28) 
    function create_if_block_2(ctx) {
    	let polygon;
    	let polygon_points_value;

    	const block = {
    		c: function create() {
    			polygon = svg_element("polygon");
    			attr_dev(polygon, "points", polygon_points_value = "" + (/*x*/ ctx[0] - RADIUS + "," + (/*y*/ ctx[1] + RADIUS) + " " + /*x*/ ctx[0] + "," + (/*y*/ ctx[1] - RADIUS) + " " + (/*x*/ ctx[0] + RADIUS) + "," + (/*y*/ ctx[1] + RADIUS)));
    			attr_dev(polygon, "class", "value svelte-lvf6t1");
    			add_location(polygon, file$5, 15, 4, 383);
    		},
    		m: function mount(target, anchor) {
    			insert_dev(target, polygon, anchor);
    		},
    		p: function update(ctx, dirty) {
    			if (dirty & /*x, y*/ 3 && polygon_points_value !== (polygon_points_value = "" + (/*x*/ ctx[0] - RADIUS + "," + (/*y*/ ctx[1] + RADIUS) + " " + /*x*/ ctx[0] + "," + (/*y*/ ctx[1] - RADIUS) + " " + (/*x*/ ctx[0] + RADIUS) + "," + (/*y*/ ctx[1] + RADIUS)))) {
    				attr_dev(polygon, "points", polygon_points_value);
    			}
    		},
    		d: function destroy(detaching) {
    			if (detaching) detach_dev(polygon);
    		}
    	};

    	dispatch_dev("SvelteRegisterBlock", {
    		block,
    		id: create_if_block_2.name,
    		type: "if",
    		source: "(15:28) ",
    		ctx
    	});

    	return block;
    }

    // (13:27) 
    function create_if_block_1$1(ctx) {
    	let rect;
    	let rect_x_value;
    	let rect_y_value;

    	const block = {
    		c: function create() {
    			rect = svg_element("rect");
    			attr_dev(rect, "x", rect_x_value = /*x*/ ctx[0] - RADIUS);
    			attr_dev(rect, "y", rect_y_value = /*y*/ ctx[1] - RADIUS);
    			attr_dev(rect, "width", RADIUS * 2);
    			attr_dev(rect, "height", RADIUS * 2);
    			attr_dev(rect, "class", "midi svelte-lvf6t1");
    			add_location(rect, file$5, 13, 4, 259);
    		},
    		m: function mount(target, anchor) {
    			insert_dev(target, rect, anchor);
    		},
    		p: function update(ctx, dirty) {
    			if (dirty & /*x*/ 1 && rect_x_value !== (rect_x_value = /*x*/ ctx[0] - RADIUS)) {
    				attr_dev(rect, "x", rect_x_value);
    			}

    			if (dirty & /*y*/ 2 && rect_y_value !== (rect_y_value = /*y*/ ctx[1] - RADIUS)) {
    				attr_dev(rect, "y", rect_y_value);
    			}
    		},
    		d: function destroy(detaching) {
    			if (detaching) detach_dev(rect);
    		}
    	};

    	dispatch_dev("SvelteRegisterBlock", {
    		block,
    		id: create_if_block_1$1.name,
    		type: "if",
    		source: "(13:27) ",
    		ctx
    	});

    	return block;
    }

    // (11:0) {#if type === "Stream"}
    function create_if_block$2(ctx) {
    	let circle;

    	const block = {
    		c: function create() {
    			circle = svg_element("circle");
    			attr_dev(circle, "cx", /*x*/ ctx[0]);
    			attr_dev(circle, "cy", /*y*/ ctx[1]);
    			attr_dev(circle, "r", RADIUS);
    			attr_dev(circle, "class", "socket svelte-lvf6t1");
    			add_location(circle, file$5, 11, 4, 176);
    		},
    		m: function mount(target, anchor) {
    			insert_dev(target, circle, anchor);
    		},
    		p: function update(ctx, dirty) {
    			if (dirty & /*x*/ 1) {
    				attr_dev(circle, "cx", /*x*/ ctx[0]);
    			}

    			if (dirty & /*y*/ 2) {
    				attr_dev(circle, "cy", /*y*/ ctx[1]);
    			}
    		},
    		d: function destroy(detaching) {
    			if (detaching) detach_dev(circle);
    		}
    	};

    	dispatch_dev("SvelteRegisterBlock", {
    		block,
    		id: create_if_block$2.name,
    		type: "if",
    		source: "(11:0) {#if type === \\\"Stream\\\"}",
    		ctx
    	});

    	return block;
    }

    function create_fragment$5(ctx) {
    	let if_block_anchor;

    	function select_block_type(ctx, dirty) {
    		if (/*type*/ ctx[2] === "Stream") return create_if_block$2;
    		if (/*type*/ ctx[2] === "Midi") return create_if_block_1$1;
    		if (/*type*/ ctx[2] === "Value") return create_if_block_2;
    	}

    	let current_block_type = select_block_type(ctx);
    	let if_block = current_block_type && current_block_type(ctx);

    	const block = {
    		c: function create() {
    			if (if_block) if_block.c();
    			if_block_anchor = empty();
    		},
    		l: function claim(nodes) {
    			throw new Error("options.hydrate only works if the component was compiled with the `hydratable: true` option");
    		},
    		m: function mount(target, anchor) {
    			if (if_block) if_block.m(target, anchor);
    			insert_dev(target, if_block_anchor, anchor);
    		},
    		p: function update(ctx, [dirty]) {
    			if (current_block_type === (current_block_type = select_block_type(ctx)) && if_block) {
    				if_block.p(ctx, dirty);
    			} else {
    				if (if_block) if_block.d(1);
    				if_block = current_block_type && current_block_type(ctx);

    				if (if_block) {
    					if_block.c();
    					if_block.m(if_block_anchor.parentNode, if_block_anchor);
    				}
    			}
    		},
    		i: noop,
    		o: noop,
    		d: function destroy(detaching) {
    			if (if_block) {
    				if_block.d(detaching);
    			}

    			if (detaching) detach_dev(if_block_anchor);
    		}
    	};

    	dispatch_dev("SvelteRegisterBlock", {
    		block,
    		id: create_fragment$5.name,
    		type: "component",
    		source: "",
    		ctx
    	});

    	return block;
    }

    const RADIUS = 12;

    function instance$5($$self, $$props, $$invalidate) {
    	let { $$slots: slots = {}, $$scope } = $$props;
    	validate_slots('Socket', slots, []);
    	let { x = 300 } = $$props;
    	let { y = 300 } = $$props;
    	let { type = "Stream" } = $$props;
    	console.log(type);
    	const writable_props = ['x', 'y', 'type'];

    	Object.keys($$props).forEach(key => {
    		if (!~writable_props.indexOf(key) && key.slice(0, 2) !== '$$' && key !== 'slot') console_1$1.warn(`<Socket> was created with unknown prop '${key}'`);
    	});

    	$$self.$$set = $$props => {
    		if ('x' in $$props) $$invalidate(0, x = $$props.x);
    		if ('y' in $$props) $$invalidate(1, y = $$props.y);
    		if ('type' in $$props) $$invalidate(2, type = $$props.type);
    	};

    	$$self.$capture_state = () => ({ RADIUS, x, y, type });

    	$$self.$inject_state = $$props => {
    		if ('x' in $$props) $$invalidate(0, x = $$props.x);
    		if ('y' in $$props) $$invalidate(1, y = $$props.y);
    		if ('type' in $$props) $$invalidate(2, type = $$props.type);
    	};

    	if ($$props && "$$inject" in $$props) {
    		$$self.$inject_state($$props.$$inject);
    	}

    	return [x, y, type];
    }

    class Socket extends SvelteComponentDev {
    	constructor(options) {
    		super(options);
    		init(this, options, instance$5, create_fragment$5, safe_not_equal, { x: 0, y: 1, type: 2 });

    		dispatch_dev("SvelteRegisterComponent", {
    			component: this,
    			tagName: "Socket",
    			options,
    			id: create_fragment$5.name
    		});
    	}

    	get x() {
    		throw new Error("<Socket>: Props cannot be read directly from the component instance unless compiling with 'accessors: true' or '<svelte:options accessors/>'");
    	}

    	set x(value) {
    		throw new Error("<Socket>: Props cannot be set directly on the component instance unless compiling with 'accessors: true' or '<svelte:options accessors/>'");
    	}

    	get y() {
    		throw new Error("<Socket>: Props cannot be read directly from the component instance unless compiling with 'accessors: true' or '<svelte:options accessors/>'");
    	}

    	set y(value) {
    		throw new Error("<Socket>: Props cannot be set directly on the component instance unless compiling with 'accessors: true' or '<svelte:options accessors/>'");
    	}

    	get type() {
    		throw new Error("<Socket>: Props cannot be read directly from the component instance unless compiling with 'accessors: true' or '<svelte:options accessors/>'");
    	}

    	set type(value) {
    		throw new Error("<Socket>: Props cannot be set directly on the component instance unless compiling with 'accessors: true' or '<svelte:options accessors/>'");
    	}
    }

    // TODO: verify this doesn't leak memory
    function storeWatcher(store) {
        this.store = store;
        this.value;

        this.store.subscribe(val => {
            this.value = val;
        });
    }

    storeWatcher.prototype.get = function() {
        return this.value;
    };

    /* src/node-editor/Node.svelte generated by Svelte v3.44.3 */
    const file$4 = "src/node-editor/Node.svelte";

    function get_each_context(ctx, list, i) {
    	const child_ctx = ctx.slice();
    	child_ctx[15] = list[i];
    	child_ctx[17] = i;
    	return child_ctx;
    }

    // (76:4) {:else}
    function create_else_block(ctx) {
    	let text_1;
    	let t_value = /*property*/ ctx[15][0] + "";
    	let t;
    	let text_1_x_value;
    	let text_1_y_value;
    	let socket;
    	let current;
    	let mounted;
    	let dispose;

    	socket = new Socket({
    			props: {
    				x: /*width*/ ctx[4],
    				y: SOCKET_LIST_START + /*SOCKET_VERTICAL_SPACING*/ ctx[6] * /*i*/ ctx[17],
    				type: /*property*/ ctx[15][1].type
    			},
    			$$inline: true
    		});

    	const block = {
    		c: function create() {
    			text_1 = svg_element("text");
    			t = text(t_value);
    			create_component(socket.$$.fragment);
    			attr_dev(text_1, "x", text_1_x_value = /*width*/ ctx[4] - TEXT_PADDING);
    			attr_dev(text_1, "y", text_1_y_value = SOCKET_LIST_START + /*SOCKET_VERTICAL_SPACING*/ ctx[6] * /*i*/ ctx[17]);
    			attr_dev(text_1, "class", "right-align svelte-1k3p7xa");
    			add_location(text_1, file$4, 76, 8, 2226);
    		},
    		m: function mount(target, anchor) {
    			insert_dev(target, text_1, anchor);
    			append_dev(text_1, t);
    			mount_component(socket, target, anchor);
    			current = true;

    			if (!mounted) {
    				dispose = listen_dev(text_1, "mousedown", /*clicked*/ ctx[8], false, false, false);
    				mounted = true;
    			}
    		},
    		p: function update(ctx, dirty) {
    			if ((!current || dirty & /*properties*/ 8) && t_value !== (t_value = /*property*/ ctx[15][0] + "")) set_data_dev(t, t_value);

    			if (!current || dirty & /*width*/ 16 && text_1_x_value !== (text_1_x_value = /*width*/ ctx[4] - TEXT_PADDING)) {
    				attr_dev(text_1, "x", text_1_x_value);
    			}

    			if (!current || dirty & /*properties*/ 8 && text_1_y_value !== (text_1_y_value = SOCKET_LIST_START + /*SOCKET_VERTICAL_SPACING*/ ctx[6] * /*i*/ ctx[17])) {
    				attr_dev(text_1, "y", text_1_y_value);
    			}

    			const socket_changes = {};
    			if (dirty & /*width*/ 16) socket_changes.x = /*width*/ ctx[4];
    			if (dirty & /*properties*/ 8) socket_changes.y = SOCKET_LIST_START + /*SOCKET_VERTICAL_SPACING*/ ctx[6] * /*i*/ ctx[17];
    			if (dirty & /*properties*/ 8) socket_changes.type = /*property*/ ctx[15][1].type;
    			socket.$set(socket_changes);
    		},
    		i: function intro(local) {
    			if (current) return;
    			transition_in(socket.$$.fragment, local);
    			current = true;
    		},
    		o: function outro(local) {
    			transition_out(socket.$$.fragment, local);
    			current = false;
    		},
    		d: function destroy(detaching) {
    			if (detaching) detach_dev(text_1);
    			destroy_component(socket, detaching);
    			mounted = false;
    			dispose();
    		}
    	};

    	dispatch_dev("SvelteRegisterBlock", {
    		block,
    		id: create_else_block.name,
    		type: "else",
    		source: "(76:4) {:else}",
    		ctx
    	});

    	return block;
    }

    // (73:4) {#if property[2] === "INPUT"}
    function create_if_block$1(ctx) {
    	let text_1;
    	let t_value = /*property*/ ctx[15][0] + "";
    	let t;
    	let text_1_y_value;
    	let socket;
    	let current;
    	let mounted;
    	let dispose;

    	socket = new Socket({
    			props: {
    				x: 0,
    				y: SOCKET_LIST_START + /*SOCKET_VERTICAL_SPACING*/ ctx[6] * /*i*/ ctx[17],
    				type: /*property*/ ctx[15][1].type
    			},
    			$$inline: true
    		});

    	const block = {
    		c: function create() {
    			text_1 = svg_element("text");
    			t = text(t_value);
    			create_component(socket.$$.fragment);
    			attr_dev(text_1, "x", TEXT_PADDING);
    			attr_dev(text_1, "y", text_1_y_value = SOCKET_LIST_START + /*SOCKET_VERTICAL_SPACING*/ ctx[6] * /*i*/ ctx[17]);
    			attr_dev(text_1, "class", "svelte-1k3p7xa");
    			add_location(text_1, file$4, 73, 8, 1986);
    		},
    		m: function mount(target, anchor) {
    			insert_dev(target, text_1, anchor);
    			append_dev(text_1, t);
    			mount_component(socket, target, anchor);
    			current = true;

    			if (!mounted) {
    				dispose = listen_dev(text_1, "mousedown", /*clicked*/ ctx[8], false, false, false);
    				mounted = true;
    			}
    		},
    		p: function update(ctx, dirty) {
    			if ((!current || dirty & /*properties*/ 8) && t_value !== (t_value = /*property*/ ctx[15][0] + "")) set_data_dev(t, t_value);

    			if (!current || dirty & /*properties*/ 8 && text_1_y_value !== (text_1_y_value = SOCKET_LIST_START + /*SOCKET_VERTICAL_SPACING*/ ctx[6] * /*i*/ ctx[17])) {
    				attr_dev(text_1, "y", text_1_y_value);
    			}

    			const socket_changes = {};
    			if (dirty & /*properties*/ 8) socket_changes.y = SOCKET_LIST_START + /*SOCKET_VERTICAL_SPACING*/ ctx[6] * /*i*/ ctx[17];
    			if (dirty & /*properties*/ 8) socket_changes.type = /*property*/ ctx[15][1].type;
    			socket.$set(socket_changes);
    		},
    		i: function intro(local) {
    			if (current) return;
    			transition_in(socket.$$.fragment, local);
    			current = true;
    		},
    		o: function outro(local) {
    			transition_out(socket.$$.fragment, local);
    			current = false;
    		},
    		d: function destroy(detaching) {
    			if (detaching) detach_dev(text_1);
    			destroy_component(socket, detaching);
    			mounted = false;
    			dispose();
    		}
    	};

    	dispatch_dev("SvelteRegisterBlock", {
    		block,
    		id: create_if_block$1.name,
    		type: "if",
    		source: "(73:4) {#if property[2] === \\\"INPUT\\\"}",
    		ctx
    	});

    	return block;
    }

    // (72:0) {#each properties as property, i (property[0])}
    function create_each_block(key_1, ctx) {
    	let first;
    	let current_block_type_index;
    	let if_block;
    	let if_block_anchor;
    	let current;
    	const if_block_creators = [create_if_block$1, create_else_block];
    	const if_blocks = [];

    	function select_block_type(ctx, dirty) {
    		if (/*property*/ ctx[15][2] === "INPUT") return 0;
    		return 1;
    	}

    	current_block_type_index = select_block_type(ctx);
    	if_block = if_blocks[current_block_type_index] = if_block_creators[current_block_type_index](ctx);

    	const block = {
    		key: key_1,
    		first: null,
    		c: function create() {
    			first = empty();
    			if_block.c();
    			if_block_anchor = empty();
    			this.first = first;
    		},
    		m: function mount(target, anchor) {
    			insert_dev(target, first, anchor);
    			if_blocks[current_block_type_index].m(target, anchor);
    			insert_dev(target, if_block_anchor, anchor);
    			current = true;
    		},
    		p: function update(new_ctx, dirty) {
    			ctx = new_ctx;
    			let previous_block_index = current_block_type_index;
    			current_block_type_index = select_block_type(ctx);

    			if (current_block_type_index === previous_block_index) {
    				if_blocks[current_block_type_index].p(ctx, dirty);
    			} else {
    				group_outros();

    				transition_out(if_blocks[previous_block_index], 1, 1, () => {
    					if_blocks[previous_block_index] = null;
    				});

    				check_outros();
    				if_block = if_blocks[current_block_type_index];

    				if (!if_block) {
    					if_block = if_blocks[current_block_type_index] = if_block_creators[current_block_type_index](ctx);
    					if_block.c();
    				} else {
    					if_block.p(ctx, dirty);
    				}

    				transition_in(if_block, 1);
    				if_block.m(if_block_anchor.parentNode, if_block_anchor);
    			}
    		},
    		i: function intro(local) {
    			if (current) return;
    			transition_in(if_block);
    			current = true;
    		},
    		o: function outro(local) {
    			transition_out(if_block);
    			current = false;
    		},
    		d: function destroy(detaching) {
    			if (detaching) detach_dev(first);
    			if_blocks[current_block_type_index].d(detaching);
    			if (detaching) detach_dev(if_block_anchor);
    		}
    	};

    	dispatch_dev("SvelteRegisterBlock", {
    		block,
    		id: create_each_block.name,
    		type: "each",
    		source: "(72:0) {#each properties as property, i (property[0])}",
    		ctx
    	});

    	return block;
    }

    function create_fragment$4(ctx) {
    	let g;
    	let rect;
    	let text_1;
    	let t;
    	let each_blocks = [];
    	let each_1_lookup = new Map();
    	let g_transform_value;
    	let current;
    	let mounted;
    	let dispose;
    	let each_value = /*properties*/ ctx[3];
    	validate_each_argument(each_value);
    	const get_key = ctx => /*property*/ ctx[15][0];
    	validate_each_keys(ctx, each_value, get_each_context, get_key);

    	for (let i = 0; i < each_value.length; i += 1) {
    		let child_ctx = get_each_context(ctx, each_value, i);
    		let key = get_key(child_ctx);
    		each_1_lookup.set(key, each_blocks[i] = create_each_block(key, child_ctx));
    	}

    	const block = {
    		c: function create() {
    			g = svg_element("g");
    			rect = svg_element("rect");
    			text_1 = svg_element("text");
    			t = text(/*title*/ ctx[2]);

    			for (let i = 0; i < each_blocks.length; i += 1) {
    				each_blocks[i].c();
    			}

    			attr_dev(rect, "width", /*width*/ ctx[4]);
    			attr_dev(rect, "height", /*computedHeight*/ ctx[7]);
    			attr_dev(rect, "rx", ROUNDNESS);
    			attr_dev(rect, "class", "background svelte-1k3p7xa");
    			add_location(rect, file$4, 68, 0, 1699);
    			attr_dev(text_1, "x", PADDING);
    			attr_dev(text_1, "y", /*PADDING_TOP*/ ctx[5]);
    			attr_dev(text_1, "class", "title svelte-1k3p7xa");
    			add_location(text_1, file$4, 69, 0, 1809);
    			attr_dev(g, "transform", g_transform_value = "translate(" + /*x*/ ctx[0] + ", " + /*y*/ ctx[1] + ")");
    			add_location(g, file$4, 67, 0, 1663);
    		},
    		l: function claim(nodes) {
    			throw new Error("options.hydrate only works if the component was compiled with the `hydratable: true` option");
    		},
    		m: function mount(target, anchor) {
    			insert_dev(target, g, anchor);
    			append_dev(g, rect);
    			append_dev(g, text_1);
    			append_dev(text_1, t);

    			for (let i = 0; i < each_blocks.length; i += 1) {
    				each_blocks[i].m(g, null);
    			}

    			current = true;

    			if (!mounted) {
    				dispose = [
    					listen_dev(rect, "mousedown", /*clicked*/ ctx[8], false, false, false),
    					listen_dev(text_1, "mousedown", /*clicked*/ ctx[8], false, false, false)
    				];

    				mounted = true;
    			}
    		},
    		p: function update(ctx, [dirty]) {
    			if (!current || dirty & /*width*/ 16) {
    				attr_dev(rect, "width", /*width*/ ctx[4]);
    			}

    			if (!current || dirty & /*title*/ 4) set_data_dev(t, /*title*/ ctx[2]);

    			if (dirty & /*SOCKET_LIST_START, SOCKET_VERTICAL_SPACING, properties, TEXT_PADDING, clicked, width*/ 344) {
    				each_value = /*properties*/ ctx[3];
    				validate_each_argument(each_value);
    				group_outros();
    				validate_each_keys(ctx, each_value, get_each_context, get_key);
    				each_blocks = update_keyed_each(each_blocks, dirty, get_key, 1, ctx, each_value, each_1_lookup, g, outro_and_destroy_block, create_each_block, null, get_each_context);
    				check_outros();
    			}

    			if (!current || dirty & /*x, y*/ 3 && g_transform_value !== (g_transform_value = "translate(" + /*x*/ ctx[0] + ", " + /*y*/ ctx[1] + ")")) {
    				attr_dev(g, "transform", g_transform_value);
    			}
    		},
    		i: function intro(local) {
    			if (current) return;

    			for (let i = 0; i < each_value.length; i += 1) {
    				transition_in(each_blocks[i]);
    			}

    			current = true;
    		},
    		o: function outro(local) {
    			for (let i = 0; i < each_blocks.length; i += 1) {
    				transition_out(each_blocks[i]);
    			}

    			current = false;
    		},
    		d: function destroy(detaching) {
    			if (detaching) detach_dev(g);

    			for (let i = 0; i < each_blocks.length; i += 1) {
    				each_blocks[i].d();
    			}

    			mounted = false;
    			run_all(dispose);
    		}
    	};

    	dispatch_dev("SvelteRegisterBlock", {
    		block,
    		id: create_fragment$4.name,
    		type: "component",
    		source: "",
    		ctx
    	});

    	return block;
    }

    const ROUNDNESS = 7;
    const PADDING = 10;
    const TEXT_PADDING = 30;
    const SOCKET_LIST_START = 55;
    const TEXT_SIZE = 14;

    function instance$4($$self, $$props, $$invalidate) {
    	let { $$slots: slots = {}, $$scope } = $$props;
    	validate_slots('Node', slots, []);
    	const PADDING_TOP = PADDING + 7;
    	const SOCKET_VERTICAL_SPACING = TEXT_SIZE + 5;
    	let { title = "Test title" } = $$props;

    	let { properties = [
    		[
    			"Audio in",
    			{
    				"type": "Stream",
    				"content": [{ "type": "Audio" }]
    			},
    			"INPUT"
    		],
    		[
    			"Audio out",
    			{
    				"type": "Value",
    				"content": [{ "type": "Audio" }]
    			},
    			"OUTPUT"
    		]
    	] } = $$props;

    	let { width = 200 } = $$props;
    	let { x = 100 } = $$props;
    	let { y = 100 } = $$props;
    	let { mouseStore } = $$props;
    	let { viewportStore } = $$props;
    	let viewport = new storeWatcher(viewportStore);
    	let computedHeight = SOCKET_LIST_START + SOCKET_VERTICAL_SPACING * (properties.length - 1) + TEXT_SIZE + PADDING;
    	let dragging = false;
    	let dragAnchor = { x: 0, y: 0 };

    	function clicked({ clientX, clientY }) {
    		dragging = true;

    		dragAnchor = {
    			x: clientX - x - viewport.get().left,
    			y: clientY - y - viewport.get().top
    		};
    	}

    	function released() {
    		dragging = false;
    	}

    	mouseStore.subscribe(([mouseX, mouseY]) => {
    		if (dragging) {
    			$$invalidate(0, x = mouseX - dragAnchor.x);
    			$$invalidate(1, y = mouseY - dragAnchor.y);
    		}
    	});

    	onMount(() => {
    		document.addEventListener("mouseup", released);
    	});

    	const writable_props = ['title', 'properties', 'width', 'x', 'y', 'mouseStore', 'viewportStore'];

    	Object.keys($$props).forEach(key => {
    		if (!~writable_props.indexOf(key) && key.slice(0, 2) !== '$$' && key !== 'slot') console.warn(`<Node> was created with unknown prop '${key}'`);
    	});

    	$$self.$$set = $$props => {
    		if ('title' in $$props) $$invalidate(2, title = $$props.title);
    		if ('properties' in $$props) $$invalidate(3, properties = $$props.properties);
    		if ('width' in $$props) $$invalidate(4, width = $$props.width);
    		if ('x' in $$props) $$invalidate(0, x = $$props.x);
    		if ('y' in $$props) $$invalidate(1, y = $$props.y);
    		if ('mouseStore' in $$props) $$invalidate(9, mouseStore = $$props.mouseStore);
    		if ('viewportStore' in $$props) $$invalidate(10, viewportStore = $$props.viewportStore);
    	};

    	$$self.$capture_state = () => ({
    		Socket,
    		onMount,
    		storeWatcher,
    		ROUNDNESS,
    		PADDING,
    		PADDING_TOP,
    		TEXT_PADDING,
    		SOCKET_LIST_START,
    		TEXT_SIZE,
    		SOCKET_VERTICAL_SPACING,
    		title,
    		properties,
    		width,
    		x,
    		y,
    		mouseStore,
    		viewportStore,
    		viewport,
    		computedHeight,
    		dragging,
    		dragAnchor,
    		clicked,
    		released
    	});

    	$$self.$inject_state = $$props => {
    		if ('title' in $$props) $$invalidate(2, title = $$props.title);
    		if ('properties' in $$props) $$invalidate(3, properties = $$props.properties);
    		if ('width' in $$props) $$invalidate(4, width = $$props.width);
    		if ('x' in $$props) $$invalidate(0, x = $$props.x);
    		if ('y' in $$props) $$invalidate(1, y = $$props.y);
    		if ('mouseStore' in $$props) $$invalidate(9, mouseStore = $$props.mouseStore);
    		if ('viewportStore' in $$props) $$invalidate(10, viewportStore = $$props.viewportStore);
    		if ('viewport' in $$props) viewport = $$props.viewport;
    		if ('computedHeight' in $$props) $$invalidate(7, computedHeight = $$props.computedHeight);
    		if ('dragging' in $$props) dragging = $$props.dragging;
    		if ('dragAnchor' in $$props) dragAnchor = $$props.dragAnchor;
    	};

    	if ($$props && "$$inject" in $$props) {
    		$$self.$inject_state($$props.$$inject);
    	}

    	return [
    		x,
    		y,
    		title,
    		properties,
    		width,
    		PADDING_TOP,
    		SOCKET_VERTICAL_SPACING,
    		computedHeight,
    		clicked,
    		mouseStore,
    		viewportStore
    	];
    }

    class Node extends SvelteComponentDev {
    	constructor(options) {
    		super(options);

    		init(this, options, instance$4, create_fragment$4, safe_not_equal, {
    			title: 2,
    			properties: 3,
    			width: 4,
    			x: 0,
    			y: 1,
    			mouseStore: 9,
    			viewportStore: 10
    		});

    		dispatch_dev("SvelteRegisterComponent", {
    			component: this,
    			tagName: "Node",
    			options,
    			id: create_fragment$4.name
    		});

    		const { ctx } = this.$$;
    		const props = options.props || {};

    		if (/*mouseStore*/ ctx[9] === undefined && !('mouseStore' in props)) {
    			console.warn("<Node> was created without expected prop 'mouseStore'");
    		}

    		if (/*viewportStore*/ ctx[10] === undefined && !('viewportStore' in props)) {
    			console.warn("<Node> was created without expected prop 'viewportStore'");
    		}
    	}

    	get title() {
    		throw new Error("<Node>: Props cannot be read directly from the component instance unless compiling with 'accessors: true' or '<svelte:options accessors/>'");
    	}

    	set title(value) {
    		throw new Error("<Node>: Props cannot be set directly on the component instance unless compiling with 'accessors: true' or '<svelte:options accessors/>'");
    	}

    	get properties() {
    		throw new Error("<Node>: Props cannot be read directly from the component instance unless compiling with 'accessors: true' or '<svelte:options accessors/>'");
    	}

    	set properties(value) {
    		throw new Error("<Node>: Props cannot be set directly on the component instance unless compiling with 'accessors: true' or '<svelte:options accessors/>'");
    	}

    	get width() {
    		throw new Error("<Node>: Props cannot be read directly from the component instance unless compiling with 'accessors: true' or '<svelte:options accessors/>'");
    	}

    	set width(value) {
    		throw new Error("<Node>: Props cannot be set directly on the component instance unless compiling with 'accessors: true' or '<svelte:options accessors/>'");
    	}

    	get x() {
    		throw new Error("<Node>: Props cannot be read directly from the component instance unless compiling with 'accessors: true' or '<svelte:options accessors/>'");
    	}

    	set x(value) {
    		throw new Error("<Node>: Props cannot be set directly on the component instance unless compiling with 'accessors: true' or '<svelte:options accessors/>'");
    	}

    	get y() {
    		throw new Error("<Node>: Props cannot be read directly from the component instance unless compiling with 'accessors: true' or '<svelte:options accessors/>'");
    	}

    	set y(value) {
    		throw new Error("<Node>: Props cannot be set directly on the component instance unless compiling with 'accessors: true' or '<svelte:options accessors/>'");
    	}

    	get mouseStore() {
    		throw new Error("<Node>: Props cannot be read directly from the component instance unless compiling with 'accessors: true' or '<svelte:options accessors/>'");
    	}

    	set mouseStore(value) {
    		throw new Error("<Node>: Props cannot be set directly on the component instance unless compiling with 'accessors: true' or '<svelte:options accessors/>'");
    	}

    	get viewportStore() {
    		throw new Error("<Node>: Props cannot be read directly from the component instance unless compiling with 'accessors: true' or '<svelte:options accessors/>'");
    	}

    	set viewportStore(value) {
    		throw new Error("<Node>: Props cannot be set directly on the component instance unless compiling with 'accessors: true' or '<svelte:options accessors/>'");
    	}
    }

    const subscriber_queue = [];
    /**
     * Create a `Writable` store that allows both updating and reading by subscription.
     * @param {*=}value initial value
     * @param {StartStopNotifier=}start start and stop notifications for subscriptions
     */
    function writable(value, start = noop) {
        let stop;
        const subscribers = new Set();
        function set(new_value) {
            if (safe_not_equal(value, new_value)) {
                value = new_value;
                if (stop) { // store is ready
                    const run_queue = !subscriber_queue.length;
                    for (const subscriber of subscribers) {
                        subscriber[1]();
                        subscriber_queue.push(subscriber, value);
                    }
                    if (run_queue) {
                        for (let i = 0; i < subscriber_queue.length; i += 2) {
                            subscriber_queue[i][0](subscriber_queue[i + 1]);
                        }
                        subscriber_queue.length = 0;
                    }
                }
            }
        }
        function update(fn) {
            set(fn(value));
        }
        function subscribe(run, invalidate = noop) {
            const subscriber = [run, invalidate];
            subscribers.add(subscriber);
            if (subscribers.size === 1) {
                stop = start(set) || noop;
            }
            run(value);
            return () => {
                subscribers.delete(subscriber);
                if (subscribers.size === 0) {
                    stop();
                    stop = null;
                }
            };
        }
        return { set, update, subscribe };
    }

    /* src/node-editor/Editor.svelte generated by Svelte v3.44.3 */

    const { Object: Object_1, console: console_1 } = globals;
    const file$3 = "src/node-editor/Editor.svelte";

    function create_fragment$3(ctx) {
    	let svg;
    	let rect;
    	let node;
    	let svg_viewBox_value;
    	let current;
    	let mounted;
    	let dispose;

    	node = new Node({
    			props: {
    				mouseStore: /*mouseMoveStore*/ ctx[5],
    				viewportStore: /*viewportStore*/ ctx[6]
    			},
    			$$inline: true
    		});

    	const block = {
    		c: function create() {
    			svg = svg_element("svg");
    			rect = svg_element("rect");
    			create_component(node.$$.fragment);
    			attr_dev(rect, "x", "-10000000");
    			attr_dev(rect, "y", "-10000000");
    			attr_dev(rect, "width", "20000000");
    			attr_dev(rect, "height", "20000000");
    			attr_dev(rect, "opacity", "0");
    			add_location(rect, file$3, 57, 4, 1887);
    			attr_dev(svg, "viewBox", svg_viewBox_value = "" + (/*viewportLeft*/ ctx[1] + " " + /*viewportTop*/ ctx[2] + " " + /*viewportWidth*/ ctx[3] + " " + /*viewportHeight*/ ctx[4]));
    			attr_dev(svg, "class", "svelte-1orbn9j");
    			add_location(svg, file$3, 55, 0, 1707);
    		},
    		l: function claim(nodes) {
    			throw new Error("options.hydrate only works if the component was compiled with the `hydratable: true` option");
    		},
    		m: function mount(target, anchor) {
    			insert_dev(target, svg, anchor);
    			append_dev(svg, rect);
    			mount_component(node, svg, null);
    			/*svg_binding*/ ctx[9](svg);
    			current = true;

    			if (!mounted) {
    				dispose = listen_dev(rect, "mousedown", backgroundMousedown, false, false, false);
    				mounted = true;
    			}
    		},
    		p: function update(ctx, [dirty]) {
    			if (!current || dirty & /*viewportLeft, viewportTop, viewportWidth, viewportHeight*/ 30 && svg_viewBox_value !== (svg_viewBox_value = "" + (/*viewportLeft*/ ctx[1] + " " + /*viewportTop*/ ctx[2] + " " + /*viewportWidth*/ ctx[3] + " " + /*viewportHeight*/ ctx[4]))) {
    				attr_dev(svg, "viewBox", svg_viewBox_value);
    			}
    		},
    		i: function intro(local) {
    			if (current) return;
    			transition_in(node.$$.fragment, local);
    			current = true;
    		},
    		o: function outro(local) {
    			transition_out(node.$$.fragment, local);
    			current = false;
    		},
    		d: function destroy(detaching) {
    			if (detaching) detach_dev(svg);
    			destroy_component(node);
    			/*svg_binding*/ ctx[9](null);
    			mounted = false;
    			dispose();
    		}
    	};

    	dispatch_dev("SvelteRegisterBlock", {
    		block,
    		id: create_fragment$3.name,
    		type: "component",
    		source: "",
    		ctx
    	});

    	return block;
    }

    function backgroundMousedown() {
    	console.log("here");
    }

    function instance$3($$self, $$props, $$invalidate) {
    	let { $$slots: slots = {}, $$scope } = $$props;
    	validate_slots('Editor', slots, []);
    	let { width = 400 } = $$props;
    	let { height = 400 } = $$props;
    	let editor;
    	let mouseMoveStore = writable([0, 0]);
    	let viewportStore = writable({ left: 0, top: 0, width, height });
    	let viewportLeft, viewportTop, viewportWidth, viewportHeight;

    	viewportStore.subscribe(({ left, top, width, height }) => {
    		$$invalidate(1, viewportLeft = left);
    		$$invalidate(2, viewportTop = top);
    		$$invalidate(3, viewportWidth = width);
    		$$invalidate(4, viewportHeight = height);
    	});

    	// whenever the editor is given a new size, perform the appropriate calculations
    	// to readjust the various sub components and variables
    	function changeDimensions(width, height) {
    		if (editor && width && height) {
    			editor.setAttribute("viewBox", `0 0 ${width} ${height}`);
    			$$invalidate(0, editor.style.width = width + "px", editor);
    			$$invalidate(0, editor.style.height = height + "px", editor);
    			let boundingRect = editor.getBoundingClientRect();

    			viewportStore.set({
    				left: boundingRect.left,
    				top: boundingRect.top,
    				width,
    				height
    			});
    		}
    	}

    	onMount(async () => {
    		changeDimensions(width, height);

    		window.addEventListener("mousemove", ({ clientX, clientY }) => {
    			let boundingRect = editor.getBoundingClientRect();
    			let relativeX = clientX - boundingRect.x;
    			let relativeY = clientY - boundingRect.y;
    			mouseMoveStore.set([relativeX, relativeY]);
    		});
    	});

    	const writable_props = ['width', 'height'];

    	Object_1.keys($$props).forEach(key => {
    		if (!~writable_props.indexOf(key) && key.slice(0, 2) !== '$$' && key !== 'slot') console_1.warn(`<Editor> was created with unknown prop '${key}'`);
    	});

    	function svg_binding($$value) {
    		binding_callbacks[$$value ? 'unshift' : 'push'](() => {
    			editor = $$value;
    			$$invalidate(0, editor);
    		});
    	}

    	$$self.$$set = $$props => {
    		if ('width' in $$props) $$invalidate(7, width = $$props.width);
    		if ('height' in $$props) $$invalidate(8, height = $$props.height);
    	};

    	$$self.$capture_state = () => ({
    		Node,
    		onMount,
    		writable,
    		width,
    		height,
    		editor,
    		mouseMoveStore,
    		viewportStore,
    		viewportLeft,
    		viewportTop,
    		viewportWidth,
    		viewportHeight,
    		changeDimensions,
    		backgroundMousedown
    	});

    	$$self.$inject_state = $$props => {
    		if ('width' in $$props) $$invalidate(7, width = $$props.width);
    		if ('height' in $$props) $$invalidate(8, height = $$props.height);
    		if ('editor' in $$props) $$invalidate(0, editor = $$props.editor);
    		if ('mouseMoveStore' in $$props) $$invalidate(5, mouseMoveStore = $$props.mouseMoveStore);
    		if ('viewportStore' in $$props) $$invalidate(6, viewportStore = $$props.viewportStore);
    		if ('viewportLeft' in $$props) $$invalidate(1, viewportLeft = $$props.viewportLeft);
    		if ('viewportTop' in $$props) $$invalidate(2, viewportTop = $$props.viewportTop);
    		if ('viewportWidth' in $$props) $$invalidate(3, viewportWidth = $$props.viewportWidth);
    		if ('viewportHeight' in $$props) $$invalidate(4, viewportHeight = $$props.viewportHeight);
    	};

    	if ($$props && "$$inject" in $$props) {
    		$$self.$inject_state($$props.$$inject);
    	}

    	$$self.$$.update = () => {
    		if ($$self.$$.dirty & /*width, height*/ 384) {
    			changeDimensions(width, height);
    		}

    		if ($$self.$$.dirty & /*width, height*/ 384) {
    			viewportStore.update(lastVal => {
    				return Object.assign(Object.assign({}, lastVal), { width, height });
    			});
    		}
    	};

    	return [
    		editor,
    		viewportLeft,
    		viewportTop,
    		viewportWidth,
    		viewportHeight,
    		mouseMoveStore,
    		viewportStore,
    		width,
    		height,
    		svg_binding
    	];
    }

    class Editor extends SvelteComponentDev {
    	constructor(options) {
    		super(options);
    		init(this, options, instance$3, create_fragment$3, safe_not_equal, { width: 7, height: 8 });

    		dispatch_dev("SvelteRegisterComponent", {
    			component: this,
    			tagName: "Editor",
    			options,
    			id: create_fragment$3.name
    		});
    	}

    	get width() {
    		throw new Error("<Editor>: Props cannot be read directly from the component instance unless compiling with 'accessors: true' or '<svelte:options accessors/>'");
    	}

    	set width(value) {
    		throw new Error("<Editor>: Props cannot be set directly on the component instance unless compiling with 'accessors: true' or '<svelte:options accessors/>'");
    	}

    	get height() {
    		throw new Error("<Editor>: Props cannot be read directly from the component instance unless compiling with 'accessors: true' or '<svelte:options accessors/>'");
    	}

    	set height(value) {
    		throw new Error("<Editor>: Props cannot be set directly on the component instance unless compiling with 'accessors: true' or '<svelte:options accessors/>'");
    	}
    }

    /* src/node-editor/SideNavbar.svelte generated by Svelte v3.44.3 */

    const file$2 = "src/node-editor/SideNavbar.svelte";

    function create_fragment$2(ctx) {
    	let nav;
    	let ul;
    	let li;

    	const block = {
    		c: function create() {
    			nav = element("nav");
    			ul = element("ul");
    			li = element("li");
    			li.textContent = "Editor";
    			attr_dev(li, "class", "svelte-61x2lk");
    			add_location(li, file$2, 6, 8, 44);
    			attr_dev(ul, "class", "svelte-61x2lk");
    			add_location(ul, file$2, 5, 4, 31);
    			attr_dev(nav, "class", "svelte-61x2lk");
    			add_location(nav, file$2, 4, 0, 21);
    		},
    		l: function claim(nodes) {
    			throw new Error("options.hydrate only works if the component was compiled with the `hydratable: true` option");
    		},
    		m: function mount(target, anchor) {
    			insert_dev(target, nav, anchor);
    			append_dev(nav, ul);
    			append_dev(ul, li);
    		},
    		p: noop,
    		i: noop,
    		o: noop,
    		d: function destroy(detaching) {
    			if (detaching) detach_dev(nav);
    		}
    	};

    	dispatch_dev("SvelteRegisterBlock", {
    		block,
    		id: create_fragment$2.name,
    		type: "component",
    		source: "",
    		ctx
    	});

    	return block;
    }

    function instance$2($$self, $$props) {
    	let { $$slots: slots = {}, $$scope } = $$props;
    	validate_slots('SideNavbar', slots, []);
    	const writable_props = [];

    	Object.keys($$props).forEach(key => {
    		if (!~writable_props.indexOf(key) && key.slice(0, 2) !== '$$' && key !== 'slot') console.warn(`<SideNavbar> was created with unknown prop '${key}'`);
    	});

    	return [];
    }

    class SideNavbar extends SvelteComponentDev {
    	constructor(options) {
    		super(options);
    		init(this, options, instance$2, create_fragment$2, safe_not_equal, {});

    		dispatch_dev("SvelteRegisterComponent", {
    			component: this,
    			tagName: "SideNavbar",
    			options,
    			id: create_fragment$2.name
    		});
    	}
    }

    var SplitDirection;
    (function (SplitDirection) {
        SplitDirection[SplitDirection["VERTICAL"] = 0] = "VERTICAL";
        SplitDirection[SplitDirection["HORIZONTAL"] = 1] = "HORIZONTAL";
    })(SplitDirection || (SplitDirection = {}));

    /* src/layout/SplitView.svelte generated by Svelte v3.44.3 */
    const file$1 = "src/layout/SplitView.svelte";

    // (57:0) {#if direction === SplitDirection.VERTICAL}
    function create_if_block(ctx) {
    	let div;
    	let t0;
    	let switch_instance0;
    	let t1;
    	let switch_instance1;
    	let current;
    	let if_block = /*canResize*/ ctx[0] && create_if_block_1(ctx);

    	const switch_instance0_spread_levels = [
    		{ width: /*firstWidth*/ ctx[8] },
    		{ height: /*height*/ ctx[3] },
    		/*firstState*/ ctx[6]
    	];

    	var switch_value = /*firstPanel*/ ctx[4];

    	function switch_props(ctx) {
    		let switch_instance0_props = {};

    		for (let i = 0; i < switch_instance0_spread_levels.length; i += 1) {
    			switch_instance0_props = assign(switch_instance0_props, switch_instance0_spread_levels[i]);
    		}

    		return {
    			props: switch_instance0_props,
    			$$inline: true
    		};
    	}

    	if (switch_value) {
    		switch_instance0 = new switch_value(switch_props());
    	}

    	const switch_instance1_spread_levels = [
    		{
    			width: /*width*/ ctx[2] - /*firstWidth*/ ctx[8]
    		},
    		{ height: /*height*/ ctx[3] },
    		/*secondState*/ ctx[7]
    	];

    	var switch_value_1 = /*secondPanel*/ ctx[5];

    	function switch_props_1(ctx) {
    		let switch_instance1_props = {};

    		for (let i = 0; i < switch_instance1_spread_levels.length; i += 1) {
    			switch_instance1_props = assign(switch_instance1_props, switch_instance1_spread_levels[i]);
    		}

    		return {
    			props: switch_instance1_props,
    			$$inline: true
    		};
    	}

    	if (switch_value_1) {
    		switch_instance1 = new switch_value_1(switch_props_1());
    	}

    	const block = {
    		c: function create() {
    			div = element("div");
    			if (if_block) if_block.c();
    			t0 = space();
    			if (switch_instance0) create_component(switch_instance0.$$.fragment);
    			t1 = space();
    			if (switch_instance1) create_component(switch_instance1.$$.fragment);
    			attr_dev(div, "class", "container vertical-split svelte-dilnj0");
    			set_style(div, "width", /*width*/ ctx[2] + "px");
    			set_style(div, "height", /*height*/ ctx[3] + "px");
    			add_location(div, file$1, 57, 0, 1618);
    		},
    		m: function mount(target, anchor) {
    			insert_dev(target, div, anchor);
    			if (if_block) if_block.m(div, null);
    			append_dev(div, t0);

    			if (switch_instance0) {
    				mount_component(switch_instance0, div, null);
    			}

    			append_dev(div, t1);

    			if (switch_instance1) {
    				mount_component(switch_instance1, div, null);
    			}

    			/*div_binding*/ ctx[14](div);
    			current = true;
    		},
    		p: function update(ctx, dirty) {
    			if (/*canResize*/ ctx[0]) {
    				if (if_block) {
    					if_block.p(ctx, dirty);
    				} else {
    					if_block = create_if_block_1(ctx);
    					if_block.c();
    					if_block.m(div, t0);
    				}
    			} else if (if_block) {
    				if_block.d(1);
    				if_block = null;
    			}

    			const switch_instance0_changes = (dirty & /*firstWidth, height, firstState*/ 328)
    			? get_spread_update(switch_instance0_spread_levels, [
    					dirty & /*firstWidth*/ 256 && { width: /*firstWidth*/ ctx[8] },
    					dirty & /*height*/ 8 && { height: /*height*/ ctx[3] },
    					dirty & /*firstState*/ 64 && get_spread_object(/*firstState*/ ctx[6])
    				])
    			: {};

    			if (switch_value !== (switch_value = /*firstPanel*/ ctx[4])) {
    				if (switch_instance0) {
    					group_outros();
    					const old_component = switch_instance0;

    					transition_out(old_component.$$.fragment, 1, 0, () => {
    						destroy_component(old_component, 1);
    					});

    					check_outros();
    				}

    				if (switch_value) {
    					switch_instance0 = new switch_value(switch_props());
    					create_component(switch_instance0.$$.fragment);
    					transition_in(switch_instance0.$$.fragment, 1);
    					mount_component(switch_instance0, div, t1);
    				} else {
    					switch_instance0 = null;
    				}
    			} else if (switch_value) {
    				switch_instance0.$set(switch_instance0_changes);
    			}

    			const switch_instance1_changes = (dirty & /*width, firstWidth, height, secondState*/ 396)
    			? get_spread_update(switch_instance1_spread_levels, [
    					dirty & /*width, firstWidth*/ 260 && {
    						width: /*width*/ ctx[2] - /*firstWidth*/ ctx[8]
    					},
    					dirty & /*height*/ 8 && { height: /*height*/ ctx[3] },
    					dirty & /*secondState*/ 128 && get_spread_object(/*secondState*/ ctx[7])
    				])
    			: {};

    			if (switch_value_1 !== (switch_value_1 = /*secondPanel*/ ctx[5])) {
    				if (switch_instance1) {
    					group_outros();
    					const old_component = switch_instance1;

    					transition_out(old_component.$$.fragment, 1, 0, () => {
    						destroy_component(old_component, 1);
    					});

    					check_outros();
    				}

    				if (switch_value_1) {
    					switch_instance1 = new switch_value_1(switch_props_1());
    					create_component(switch_instance1.$$.fragment);
    					transition_in(switch_instance1.$$.fragment, 1);
    					mount_component(switch_instance1, div, null);
    				} else {
    					switch_instance1 = null;
    				}
    			} else if (switch_value_1) {
    				switch_instance1.$set(switch_instance1_changes);
    			}

    			if (!current || dirty & /*width*/ 4) {
    				set_style(div, "width", /*width*/ ctx[2] + "px");
    			}

    			if (!current || dirty & /*height*/ 8) {
    				set_style(div, "height", /*height*/ ctx[3] + "px");
    			}
    		},
    		i: function intro(local) {
    			if (current) return;
    			if (switch_instance0) transition_in(switch_instance0.$$.fragment, local);
    			if (switch_instance1) transition_in(switch_instance1.$$.fragment, local);
    			current = true;
    		},
    		o: function outro(local) {
    			if (switch_instance0) transition_out(switch_instance0.$$.fragment, local);
    			if (switch_instance1) transition_out(switch_instance1.$$.fragment, local);
    			current = false;
    		},
    		d: function destroy(detaching) {
    			if (detaching) detach_dev(div);
    			if (if_block) if_block.d();
    			if (switch_instance0) destroy_component(switch_instance0);
    			if (switch_instance1) destroy_component(switch_instance1);
    			/*div_binding*/ ctx[14](null);
    		}
    	};

    	dispatch_dev("SvelteRegisterBlock", {
    		block,
    		id: create_if_block.name,
    		type: "if",
    		source: "(57:0) {#if direction === SplitDirection.VERTICAL}",
    		ctx
    	});

    	return block;
    }

    // (59:4) {#if canResize}
    function create_if_block_1(ctx) {
    	let div1;
    	let div0;
    	let mounted;
    	let dispose;

    	const block = {
    		c: function create() {
    			div1 = element("div");
    			div0 = element("div");
    			attr_dev(div0, "class", "divider divider-vertical svelte-dilnj0");
    			set_style(div0, "left", /*firstWidth*/ ctx[8] - 2 + "px");
    			set_style(div0, "height", /*height*/ ctx[3] + "px");
    			toggle_class(div0, "dragging", /*currentlyResizingDivider*/ ctx[10]);
    			add_location(div0, file$1, 60, 12, 1793);
    			attr_dev(div1, "class", "divider-parent svelte-dilnj0");
    			add_location(div1, file$1, 59, 8, 1752);
    		},
    		m: function mount(target, anchor) {
    			insert_dev(target, div1, anchor);
    			append_dev(div1, div0);

    			if (!mounted) {
    				dispose = listen_dev(div0, "mousedown", /*dividerMousedown*/ ctx[11], false, false, false);
    				mounted = true;
    			}
    		},
    		p: function update(ctx, dirty) {
    			if (dirty & /*firstWidth*/ 256) {
    				set_style(div0, "left", /*firstWidth*/ ctx[8] - 2 + "px");
    			}

    			if (dirty & /*height*/ 8) {
    				set_style(div0, "height", /*height*/ ctx[3] + "px");
    			}

    			if (dirty & /*currentlyResizingDivider*/ 1024) {
    				toggle_class(div0, "dragging", /*currentlyResizingDivider*/ ctx[10]);
    			}
    		},
    		d: function destroy(detaching) {
    			if (detaching) detach_dev(div1);
    			mounted = false;
    			dispose();
    		}
    	};

    	dispatch_dev("SvelteRegisterBlock", {
    		block,
    		id: create_if_block_1.name,
    		type: "if",
    		source: "(59:4) {#if canResize}",
    		ctx
    	});

    	return block;
    }

    function create_fragment$1(ctx) {
    	let if_block_anchor;
    	let current;
    	let if_block = /*direction*/ ctx[1] === SplitDirection.VERTICAL && create_if_block(ctx);

    	const block = {
    		c: function create() {
    			if (if_block) if_block.c();
    			if_block_anchor = empty();
    		},
    		l: function claim(nodes) {
    			throw new Error("options.hydrate only works if the component was compiled with the `hydratable: true` option");
    		},
    		m: function mount(target, anchor) {
    			if (if_block) if_block.m(target, anchor);
    			insert_dev(target, if_block_anchor, anchor);
    			current = true;
    		},
    		p: function update(ctx, [dirty]) {
    			if (/*direction*/ ctx[1] === SplitDirection.VERTICAL) {
    				if (if_block) {
    					if_block.p(ctx, dirty);

    					if (dirty & /*direction*/ 2) {
    						transition_in(if_block, 1);
    					}
    				} else {
    					if_block = create_if_block(ctx);
    					if_block.c();
    					transition_in(if_block, 1);
    					if_block.m(if_block_anchor.parentNode, if_block_anchor);
    				}
    			} else if (if_block) {
    				group_outros();

    				transition_out(if_block, 1, 1, () => {
    					if_block = null;
    				});

    				check_outros();
    			}
    		},
    		i: function intro(local) {
    			if (current) return;
    			transition_in(if_block);
    			current = true;
    		},
    		o: function outro(local) {
    			transition_out(if_block);
    			current = false;
    		},
    		d: function destroy(detaching) {
    			if (if_block) if_block.d(detaching);
    			if (detaching) detach_dev(if_block_anchor);
    		}
    	};

    	dispatch_dev("SvelteRegisterBlock", {
    		block,
    		id: create_fragment$1.name,
    		type: "component",
    		source: "",
    		ctx
    	});

    	return block;
    }

    function instance$1($$self, $$props, $$invalidate) {
    	let { $$slots: slots = {}, $$scope } = $$props;
    	validate_slots('SplitView', slots, []);
    	let { direction } = $$props;
    	let { width } = $$props;
    	let { height } = $$props;
    	let { firstPanel } = $$props;
    	let { secondPanel } = $$props;
    	let { firstState = {} } = $$props;
    	let { secondState = {} } = $$props;
    	let { canResize = true } = $$props;
    	let { hasFixedWidth = false } = $$props;
    	let { fixedWidth = 0 } = $$props;
    	let firstWidth, firstHeight;
    	let container;
    	let currentlyResizingDivider = false;

    	function dividerMousedown(e) {
    		if (canResize) {
    			$$invalidate(10, currentlyResizingDivider = true);
    		}
    	}

    	if (!hasFixedWidth) {
    		switch (direction) {
    			case SplitDirection.VERTICAL:
    				firstWidth = Math.floor(width / 2);
    				firstHeight = height;
    				break;
    			case SplitDirection.HORIZONTAL:
    				firstHeight = Math.floor(height / 2);
    				firstWidth = width;
    				break;
    		}
    	} else {
    		firstWidth = fixedWidth;
    	}

    	onMount(async () => {
    		window.addEventListener("mousemove", e => {
    			let { clientX, clientY } = e;

    			if (currentlyResizingDivider) {
    				e.preventDefault(); // stop the text from being selected during drag
    				const containerPos = container.getBoundingClientRect();

    				if (direction === SplitDirection.VERTICAL) {
    					$$invalidate(8, firstWidth = clientX - containerPos.left);
    				}
    			}
    		});

    		window.addEventListener("mouseup", function () {
    			$$invalidate(10, currentlyResizingDivider = false);
    		});
    	});

    	const writable_props = [
    		'direction',
    		'width',
    		'height',
    		'firstPanel',
    		'secondPanel',
    		'firstState',
    		'secondState',
    		'canResize',
    		'hasFixedWidth',
    		'fixedWidth'
    	];

    	Object.keys($$props).forEach(key => {
    		if (!~writable_props.indexOf(key) && key.slice(0, 2) !== '$$' && key !== 'slot') console.warn(`<SplitView> was created with unknown prop '${key}'`);
    	});

    	function div_binding($$value) {
    		binding_callbacks[$$value ? 'unshift' : 'push'](() => {
    			container = $$value;
    			$$invalidate(9, container);
    		});
    	}

    	$$self.$$set = $$props => {
    		if ('direction' in $$props) $$invalidate(1, direction = $$props.direction);
    		if ('width' in $$props) $$invalidate(2, width = $$props.width);
    		if ('height' in $$props) $$invalidate(3, height = $$props.height);
    		if ('firstPanel' in $$props) $$invalidate(4, firstPanel = $$props.firstPanel);
    		if ('secondPanel' in $$props) $$invalidate(5, secondPanel = $$props.secondPanel);
    		if ('firstState' in $$props) $$invalidate(6, firstState = $$props.firstState);
    		if ('secondState' in $$props) $$invalidate(7, secondState = $$props.secondState);
    		if ('canResize' in $$props) $$invalidate(0, canResize = $$props.canResize);
    		if ('hasFixedWidth' in $$props) $$invalidate(12, hasFixedWidth = $$props.hasFixedWidth);
    		if ('fixedWidth' in $$props) $$invalidate(13, fixedWidth = $$props.fixedWidth);
    	};

    	$$self.$capture_state = () => ({
    		onMount,
    		SplitDirection,
    		direction,
    		width,
    		height,
    		firstPanel,
    		secondPanel,
    		firstState,
    		secondState,
    		canResize,
    		hasFixedWidth,
    		fixedWidth,
    		firstWidth,
    		firstHeight,
    		container,
    		currentlyResizingDivider,
    		dividerMousedown
    	});

    	$$self.$inject_state = $$props => {
    		if ('direction' in $$props) $$invalidate(1, direction = $$props.direction);
    		if ('width' in $$props) $$invalidate(2, width = $$props.width);
    		if ('height' in $$props) $$invalidate(3, height = $$props.height);
    		if ('firstPanel' in $$props) $$invalidate(4, firstPanel = $$props.firstPanel);
    		if ('secondPanel' in $$props) $$invalidate(5, secondPanel = $$props.secondPanel);
    		if ('firstState' in $$props) $$invalidate(6, firstState = $$props.firstState);
    		if ('secondState' in $$props) $$invalidate(7, secondState = $$props.secondState);
    		if ('canResize' in $$props) $$invalidate(0, canResize = $$props.canResize);
    		if ('hasFixedWidth' in $$props) $$invalidate(12, hasFixedWidth = $$props.hasFixedWidth);
    		if ('fixedWidth' in $$props) $$invalidate(13, fixedWidth = $$props.fixedWidth);
    		if ('firstWidth' in $$props) $$invalidate(8, firstWidth = $$props.firstWidth);
    		if ('firstHeight' in $$props) firstHeight = $$props.firstHeight;
    		if ('container' in $$props) $$invalidate(9, container = $$props.container);
    		if ('currentlyResizingDivider' in $$props) $$invalidate(10, currentlyResizingDivider = $$props.currentlyResizingDivider);
    	};

    	if ($$props && "$$inject" in $$props) {
    		$$self.$inject_state($$props.$$inject);
    	}

    	$$self.$$.update = () => {
    		if ($$self.$$.dirty & /*hasFixedWidth*/ 4096) {
    			if (hasFixedWidth) {
    				$$invalidate(0, canResize = false);
    			}
    		}
    	};

    	return [
    		canResize,
    		direction,
    		width,
    		height,
    		firstPanel,
    		secondPanel,
    		firstState,
    		secondState,
    		firstWidth,
    		container,
    		currentlyResizingDivider,
    		dividerMousedown,
    		hasFixedWidth,
    		fixedWidth,
    		div_binding
    	];
    }

    class SplitView extends SvelteComponentDev {
    	constructor(options) {
    		super(options);

    		init(this, options, instance$1, create_fragment$1, safe_not_equal, {
    			direction: 1,
    			width: 2,
    			height: 3,
    			firstPanel: 4,
    			secondPanel: 5,
    			firstState: 6,
    			secondState: 7,
    			canResize: 0,
    			hasFixedWidth: 12,
    			fixedWidth: 13
    		});

    		dispatch_dev("SvelteRegisterComponent", {
    			component: this,
    			tagName: "SplitView",
    			options,
    			id: create_fragment$1.name
    		});

    		const { ctx } = this.$$;
    		const props = options.props || {};

    		if (/*direction*/ ctx[1] === undefined && !('direction' in props)) {
    			console.warn("<SplitView> was created without expected prop 'direction'");
    		}

    		if (/*width*/ ctx[2] === undefined && !('width' in props)) {
    			console.warn("<SplitView> was created without expected prop 'width'");
    		}

    		if (/*height*/ ctx[3] === undefined && !('height' in props)) {
    			console.warn("<SplitView> was created without expected prop 'height'");
    		}

    		if (/*firstPanel*/ ctx[4] === undefined && !('firstPanel' in props)) {
    			console.warn("<SplitView> was created without expected prop 'firstPanel'");
    		}

    		if (/*secondPanel*/ ctx[5] === undefined && !('secondPanel' in props)) {
    			console.warn("<SplitView> was created without expected prop 'secondPanel'");
    		}
    	}

    	get direction() {
    		throw new Error("<SplitView>: Props cannot be read directly from the component instance unless compiling with 'accessors: true' or '<svelte:options accessors/>'");
    	}

    	set direction(value) {
    		throw new Error("<SplitView>: Props cannot be set directly on the component instance unless compiling with 'accessors: true' or '<svelte:options accessors/>'");
    	}

    	get width() {
    		throw new Error("<SplitView>: Props cannot be read directly from the component instance unless compiling with 'accessors: true' or '<svelte:options accessors/>'");
    	}

    	set width(value) {
    		throw new Error("<SplitView>: Props cannot be set directly on the component instance unless compiling with 'accessors: true' or '<svelte:options accessors/>'");
    	}

    	get height() {
    		throw new Error("<SplitView>: Props cannot be read directly from the component instance unless compiling with 'accessors: true' or '<svelte:options accessors/>'");
    	}

    	set height(value) {
    		throw new Error("<SplitView>: Props cannot be set directly on the component instance unless compiling with 'accessors: true' or '<svelte:options accessors/>'");
    	}

    	get firstPanel() {
    		throw new Error("<SplitView>: Props cannot be read directly from the component instance unless compiling with 'accessors: true' or '<svelte:options accessors/>'");
    	}

    	set firstPanel(value) {
    		throw new Error("<SplitView>: Props cannot be set directly on the component instance unless compiling with 'accessors: true' or '<svelte:options accessors/>'");
    	}

    	get secondPanel() {
    		throw new Error("<SplitView>: Props cannot be read directly from the component instance unless compiling with 'accessors: true' or '<svelte:options accessors/>'");
    	}

    	set secondPanel(value) {
    		throw new Error("<SplitView>: Props cannot be set directly on the component instance unless compiling with 'accessors: true' or '<svelte:options accessors/>'");
    	}

    	get firstState() {
    		throw new Error("<SplitView>: Props cannot be read directly from the component instance unless compiling with 'accessors: true' or '<svelte:options accessors/>'");
    	}

    	set firstState(value) {
    		throw new Error("<SplitView>: Props cannot be set directly on the component instance unless compiling with 'accessors: true' or '<svelte:options accessors/>'");
    	}

    	get secondState() {
    		throw new Error("<SplitView>: Props cannot be read directly from the component instance unless compiling with 'accessors: true' or '<svelte:options accessors/>'");
    	}

    	set secondState(value) {
    		throw new Error("<SplitView>: Props cannot be set directly on the component instance unless compiling with 'accessors: true' or '<svelte:options accessors/>'");
    	}

    	get canResize() {
    		throw new Error("<SplitView>: Props cannot be read directly from the component instance unless compiling with 'accessors: true' or '<svelte:options accessors/>'");
    	}

    	set canResize(value) {
    		throw new Error("<SplitView>: Props cannot be set directly on the component instance unless compiling with 'accessors: true' or '<svelte:options accessors/>'");
    	}

    	get hasFixedWidth() {
    		throw new Error("<SplitView>: Props cannot be read directly from the component instance unless compiling with 'accessors: true' or '<svelte:options accessors/>'");
    	}

    	set hasFixedWidth(value) {
    		throw new Error("<SplitView>: Props cannot be set directly on the component instance unless compiling with 'accessors: true' or '<svelte:options accessors/>'");
    	}

    	get fixedWidth() {
    		throw new Error("<SplitView>: Props cannot be read directly from the component instance unless compiling with 'accessors: true' or '<svelte:options accessors/>'");
    	}

    	set fixedWidth(value) {
    		throw new Error("<SplitView>: Props cannot be set directly on the component instance unless compiling with 'accessors: true' or '<svelte:options accessors/>'");
    	}
    }

    const windowDimensions = writable([window.innerWidth, window.innerHeight]);


    window.addEventListener("resize", () => {
        windowDimensions.set([window.innerWidth, window.innerHeight]);
    });

    /* src/App.svelte generated by Svelte v3.44.3 */
    const file = "src/App.svelte";

    function create_fragment(ctx) {
    	let main;
    	let splitview;
    	let current;

    	splitview = new SplitView({
    			props: {
    				direction: SplitDirection.VERTICAL,
    				width: /*width*/ ctx[0],
    				height: /*height*/ ctx[1],
    				hasFixedWidth: true,
    				fixedWidth: 48,
    				firstPanel: SideNavbar,
    				secondPanel: SplitView,
    				secondState: {
    					direction: SplitDirection.VERTICAL,
    					firstPanel: PropertyEditor,
    					secondPanel: Editor
    				}
    			},
    			$$inline: true
    		});

    	const block = {
    		c: function create() {
    			main = element("main");
    			create_component(splitview.$$.fragment);
    			add_location(main, file, 14, 0, 515);
    		},
    		l: function claim(nodes) {
    			throw new Error("options.hydrate only works if the component was compiled with the `hydratable: true` option");
    		},
    		m: function mount(target, anchor) {
    			insert_dev(target, main, anchor);
    			mount_component(splitview, main, null);
    			current = true;
    		},
    		p: function update(ctx, [dirty]) {
    			const splitview_changes = {};
    			if (dirty & /*width*/ 1) splitview_changes.width = /*width*/ ctx[0];
    			if (dirty & /*height*/ 2) splitview_changes.height = /*height*/ ctx[1];
    			splitview.$set(splitview_changes);
    		},
    		i: function intro(local) {
    			if (current) return;
    			transition_in(splitview.$$.fragment, local);
    			current = true;
    		},
    		o: function outro(local) {
    			transition_out(splitview.$$.fragment, local);
    			current = false;
    		},
    		d: function destroy(detaching) {
    			if (detaching) detach_dev(main);
    			destroy_component(splitview);
    		}
    	};

    	dispatch_dev("SvelteRegisterBlock", {
    		block,
    		id: create_fragment.name,
    		type: "component",
    		source: "",
    		ctx
    	});

    	return block;
    }

    function instance($$self, $$props, $$invalidate) {
    	let { $$slots: slots = {}, $$scope } = $$props;
    	validate_slots('App', slots, []);
    	let width = 0;
    	let height = 0;

    	windowDimensions.subscribe(([windowWidth, windowHeight]) => {
    		$$invalidate(0, width = windowWidth - 1);
    		$$invalidate(1, height = windowHeight - 3);
    	});

    	const writable_props = [];

    	Object.keys($$props).forEach(key => {
    		if (!~writable_props.indexOf(key) && key.slice(0, 2) !== '$$' && key !== 'slot') console.warn(`<App> was created with unknown prop '${key}'`);
    	});

    	$$self.$capture_state = () => ({
    		PropertyEditor,
    		Editor,
    		SideNavbar,
    		SplitView,
    		SplitDirection,
    		windowDimensions,
    		width,
    		height
    	});

    	$$self.$inject_state = $$props => {
    		if ('width' in $$props) $$invalidate(0, width = $$props.width);
    		if ('height' in $$props) $$invalidate(1, height = $$props.height);
    	};

    	if ($$props && "$$inject" in $$props) {
    		$$self.$inject_state($$props.$$inject);
    	}

    	return [width, height];
    }

    class App extends SvelteComponentDev {
    	constructor(options) {
    		super(options);
    		init(this, options, instance, create_fragment, safe_not_equal, {});

    		dispatch_dev("SvelteRegisterComponent", {
    			component: this,
    			tagName: "App",
    			options,
    			id: create_fragment.name
    		});
    	}
    }

    const app = new App({
        target: document.body,
        props: {
            name: 'World'
        }
    });

    return app;

})();
//# sourceMappingURL=bundle.js.map
