// node-bindings
#[macro_use] extern crate neon;
extern crate glutin;
extern crate gleam;
extern crate webrender;
extern crate app_units;
extern crate serde;
extern crate serde_json;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate log;
extern crate env_logger;

mod window;

use neon::prelude::*;
use window::{Window};
use std::mem::size_of;

declare_types! {
    pub class JsWindow for Window {
        init(mut ctx) {
            let title = ctx.argument::<JsString>(0)?.value();
            let width = ctx.argument::<JsNumber>(1)?.value();
            let height = ctx.argument::<JsNumber>(2)?.value();

            let w = Window::new(title, width, height);

            Ok(w)
        }

        method createBucket(mut ctx) {
            let data = ctx.argument::<JsString>(0)?.value();
            let item = serde_json::from_str(&data).unwrap();

            let index = {
                let mut this = ctx.this();
                let guard = ctx.lock();
                let mut w = this.borrow_mut(&guard);

                w.create_bucket(item)
            };

            // TODO: maybe we can restrict vector size?
            Ok(ctx.number(index as f64).upcast())
        }

        method updateBucket(mut ctx) {
            let bucket = ctx.argument::<JsNumber>(0)?.value() as usize;

            let data = ctx.argument::<JsString>(1)?.value();
            let item = serde_json::from_str(&data).unwrap();

            let mut this = ctx.this();

            ctx.borrow_mut(&mut this, |mut w| w.update_bucket(bucket, item));

            Ok(ctx.undefined().upcast())
        }

        method render(mut ctx) {
            let data = ctx.argument::<JsString>(0)?.value();
            let request = serde_json::from_str(&data).unwrap();
            let mut this = ctx.this();

            ctx.borrow_mut(&mut this, |mut w| w.render(request));

            Ok(ctx.undefined().upcast())
        }

        method handleEvents(mut ctx) {
            let mut this = ctx.this();
            // TODO: for some reason, we can't lock just once (JsArrayBuffer requires mutable ctx, ctx.lock does immutable)
            let callback_ids = ctx.borrow_mut(&mut this, |mut w| w.handle_events());

            let mut b = JsArrayBuffer::new(&mut ctx, (callback_ids.len() * size_of::<u32>()) as u32).unwrap();

            {
                let guard = ctx.lock();
                let slice = b.borrow_mut(&guard).as_mut_slice::<u32>();

                slice.copy_from_slice(&callback_ids[..]);
            }

            Ok(b.upcast())
        }

        method getGlyphIndicesAndAdvances(mut ctx) {
            let str = ctx.argument::<JsString>(0)?.value();
            let mut this = ctx.this();

            let (glyph_indices, advances) = ctx.borrow(&mut this, |w| w.get_glyph_indices_and_advances(&str));
            let len = glyph_indices.len() as u32;

            let js_array = JsArray::new(&mut ctx, 2);

            let mut b1 = JsArrayBuffer::new(&mut ctx, len * (size_of::<u32>() as u32)).unwrap();
            let mut b2 = JsArrayBuffer::new(&mut ctx, len * (size_of::<f32>() as u32)).unwrap();

            {
                let guard = ctx.lock();

                let slice = b1.borrow_mut(&guard).as_mut_slice::<u32>();
                slice.copy_from_slice(&glyph_indices[..]);

                let slice = b2.borrow_mut(&guard).as_mut_slice::<f32>();
                slice.copy_from_slice(&advances[..]);
            }

            js_array.set(&mut ctx, 0, b1).unwrap();
            js_array.set(&mut ctx, 1, b2).unwrap();

            Ok(js_array.upcast())
        }
    }
}

register_module!(mut ctx, {
    env_logger::init();

    ctx.export_class::<JsWindow>("Window")
});
