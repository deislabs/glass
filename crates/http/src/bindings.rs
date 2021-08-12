pub mod deislabs_http_v01 {
  #[allow(unused_imports)]
  use witx_bindgen_wasmtime::{wasmtime, anyhow};
  pub type HttpStatus = u16;
  pub type BodyParam<'a,> = &'a [u8];
  pub type BodyResult = Vec<u8>;
  pub type HeadersParam<'a,> = &'a [&'a  str];
  pub type HeadersResult = Vec<String>;
  pub type Params<'a,> = &'a [&'a  str];
  pub type Uri<'a,> = &'a  str;
  pub type Request<'a,> = (Method,Uri<'a,>,HeadersParam<'a,>,Option<Params<'a,>>,Option<BodyParam<'a,>>,);
  pub type Response = (HttpStatus,Option<HeadersResult>,Option<BodyResult>,);
  #[repr(u8)]
  #[derive(Clone, Copy, PartialEq, Eq)]
  pub enum Method{
    Get,
    Post,
    Put,
    Delete,
    Patch,
  }
  impl std::fmt::Debug for Method {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
      match self {
        Method::Get => {
          f.debug_tuple("Method::Get").finish()
        }
        Method::Post => {
          f.debug_tuple("Method::Post").finish()
        }
        Method::Put => {
          f.debug_tuple("Method::Put").finish()
        }
        Method::Delete => {
          f.debug_tuple("Method::Delete").finish()
        }
        Method::Patch => {
          f.debug_tuple("Method::Patch").finish()
        }
      }
    }
  }
  
  /// Auxiliary data associated with the wasm exports.
  ///
  /// This is required to be stored within the data of a
  /// `Store<T>` itself so lifting/lowering state can be managed
  /// when translating between the host and wasm.
  #[derive(Default)]
  pub struct DeislabsHttpV01Data {
  }
  pub struct DeislabsHttpV01<T> {
    get_state: Box<dyn Fn(&mut T) -> &mut DeislabsHttpV01Data + Send + Sync>,
    canonical_abi_free: wasmtime::TypedFunc<(i32, i32, i32), ()>,
    canonical_abi_realloc: wasmtime::TypedFunc<(i32, i32, i32, i32), i32>,
    handler: wasmtime::TypedFunc<(i32,i32,i32,i32,i32,i32,i32,i32,i32,i32,i32,), (i32,)>,
    memory: wasmtime::Memory,
  }
  impl<T> DeislabsHttpV01<T> {
    #[allow(unused_variables)]
    
    /// Adds any intrinsics, if necessary for this exported wasm
    /// functionality to the `linker` provided.
    ///
    /// The `get_state` closure is required to access the
    /// auxiliary data necessary for these wasm exports from
    /// the general store's state.
    pub fn add_to_linker(
    linker: &mut wasmtime::Linker<T>,
    get_state: impl Fn(&mut T) -> &mut DeislabsHttpV01Data + Send + Sync + Copy + 'static,
    ) -> anyhow::Result<()> {
      Ok(())
    }
    
    /// Instantaites the provided `module` using the specified
    /// parameters, wrapping up the result in a structure that
    /// translates between wasm and the host.
    ///
    /// The `linker` provided will have intrinsics added to it
    /// automatically, so it's not necessary to call
    /// `add_to_linker` beforehand. This function will
    /// instantiate the `module` otherwise using `linker`, and
    /// both an instance of this structure and the underlying
    /// `wasmtime::Instance` will be returned.
    ///
    /// The `get_state` parameter is used to access the
    /// auxiliary state necessary for these wasm exports from
    /// the general store state `T`.
    pub fn instantiate(
    mut store: impl wasmtime::AsContextMut<Data = T>,
    module: &wasmtime::Module,
    linker: &mut wasmtime::Linker<T>,
    get_state: impl Fn(&mut T) -> &mut DeislabsHttpV01Data + Send + Sync + Copy + 'static,
    ) -> anyhow::Result<(Self, wasmtime::Instance)> {
      Self::add_to_linker(linker, get_state)?;
      let instance = linker.instantiate(&mut store, module)?;
      Ok((Self::new(store, &instance,get_state)?, instance))
    }
    
    /// Low-level creation wrapper for wrapping up the exports
    /// of the `instance` provided in this structure of wasm
    /// exports.
    ///
    /// This function will extract exports from the `instance`
    /// defined within `store` and wrap them all up in the
    /// returned structure which can be used to interact with
    /// the wasm module.
    pub fn new(
    mut store: impl wasmtime::AsContextMut<Data = T>,
    instance: &wasmtime::Instance,
    get_state: impl Fn(&mut T) -> &mut DeislabsHttpV01Data + Send + Sync + Copy + 'static,
    ) -> anyhow::Result<Self> {
      let mut store = store.as_context_mut();
      let canonical_abi_free= instance.get_typed_func::<(i32, i32, i32), (), _>(&mut store, "canonical_abi_free")?;
      let canonical_abi_realloc= instance.get_typed_func::<(i32, i32, i32, i32), i32, _>(&mut store, "canonical_abi_realloc")?;
      let handler= instance.get_typed_func::<(i32,i32,i32,i32,i32,i32,i32,i32,i32,i32,i32,), (i32,), _>(&mut store, "handler")?;
      let memory= instance
      .get_memory(&mut store, "memory")
      .ok_or_else(|| {
        anyhow::anyhow!("`memory` export not a memory")
      })?
      ;
      Ok(DeislabsHttpV01{
        canonical_abi_free,
        canonical_abi_realloc,
        handler,
        memory,
        get_state: Box::new(get_state),
        
      })
    }
    pub fn handler(&self, mut caller: impl wasmtime::AsContextMut<Data = T>,req: Request<'_,>,)-> Result<Response, wasmtime::Trap> {
      let func_canonical_abi_free = &self.canonical_abi_free;
      let func_canonical_abi_realloc = &self.canonical_abi_realloc;
      let memory = &self.memory;
      let (t0_0, t0_1, t0_2, t0_3, t0_4, ) = req;
      let vec1 = t0_1;
      let ptr1 = func_canonical_abi_realloc.call(&mut caller, (0, 0, 1, (vec1.len() as i32) * 1))?;
      memory.data_mut(&mut caller).store_many(ptr1, vec1.as_ref())?;
      let vec3 = t0_2;
      let len3 = vec3.len() as i32;
      let result3 = func_canonical_abi_realloc.call(&mut caller, (0, 0, 4, len3 * 8))?;
      for (i, e) in vec3.into_iter().enumerate() {
        let base = result3 + (i as i32) * 8;
        {
          let vec2 = e;
          let ptr2 = func_canonical_abi_realloc.call(&mut caller, (0, 0, 1, (vec2.len() as i32) * 1))?;
          memory.data_mut(&mut caller).store_many(ptr2, vec2.as_ref())?;
          memory.data_mut(&mut caller).store(base + 4, witx_bindgen_wasmtime::rt::as_i32(vec2.len() as i32))?;
          memory.data_mut(&mut caller).store(base + 0, witx_bindgen_wasmtime::rt::as_i32(ptr2))?;
        }}let (result6_0,result6_1,result6_2,) = match t0_3{
          None => { (0i32, 0i32, 0i32)}
          Some(e) => { {
            let vec5 = e;
            let len5 = vec5.len() as i32;
            let result5 = func_canonical_abi_realloc.call(&mut caller, (0, 0, 4, len5 * 8))?;
            for (i, e) in vec5.into_iter().enumerate() {
              let base = result5 + (i as i32) * 8;
              {
                let vec4 = e;
                let ptr4 = func_canonical_abi_realloc.call(&mut caller, (0, 0, 1, (vec4.len() as i32) * 1))?;
                memory.data_mut(&mut caller).store_many(ptr4, vec4.as_ref())?;
                memory.data_mut(&mut caller).store(base + 4, witx_bindgen_wasmtime::rt::as_i32(vec4.len() as i32))?;
                memory.data_mut(&mut caller).store(base + 0, witx_bindgen_wasmtime::rt::as_i32(ptr4))?;
              }}(1i32, result5, len5)
            }}
          };
          let (result8_0,result8_1,result8_2,) = match t0_4{
            None => { (0i32, 0i32, 0i32)}
            Some(e) => { {
              let vec7 = e;
              let ptr7 = func_canonical_abi_realloc.call(&mut caller, (0, 0, 1, (vec7.len() as i32) * 1))?;
              memory.data_mut(&mut caller).store_many(ptr7, vec7.as_ref())?;
              (1i32, ptr7, vec7.len() as i32)
            }}
          };
          let (result9_0,) = self.handler.call(&mut caller, (t0_0 as i32, ptr1, vec1.len() as i32, result3, len3, result6_0, result6_1, result6_2, result8_0, result8_1, result8_2, ))?;
          let load10 = memory.data_mut(&mut caller).load::<i32>(result9_0 + 0)?;
          let load11 = memory.data_mut(&mut caller).load::<i32>(result9_0 + 8)?;
          let load12 = memory.data_mut(&mut caller).load::<i32>(result9_0 + 16)?;
          let load13 = memory.data_mut(&mut caller).load::<i32>(result9_0 + 24)?;
          let load14 = memory.data_mut(&mut caller).load::<i32>(result9_0 + 32)?;
          let load15 = memory.data_mut(&mut caller).load::<i32>(result9_0 + 40)?;
          let load16 = memory.data_mut(&mut caller).load::<i32>(result9_0 + 48)?;
          Ok((u16::try_from(load10).map_err(bad_int)?, match load11 {
            0 => None,
            1 => Some({
              let len20 = load13;
              let base20 = load12;
              let mut result20 = Vec::with_capacity(len20 as usize);
              for i in 0..len20 {
                let base = base20 + i *8;
                result20.push({
                  let load17 = memory.data_mut(&mut caller).load::<i32>(base + 0)?;
                  let load18 = memory.data_mut(&mut caller).load::<i32>(base + 4)?;
                  let ptr19 = load17;
                  let len19 = load18;
                  String::from_utf8(
                  copy_slice(
                  &mut caller,
                  memory,
                  func_canonical_abi_free,
                  ptr19, len19, 1
                  )?
                  )
                  .map_err(|_| wasmtime::Trap::new("invalid utf-8"))?
                });
              }
              func_canonical_abi_free.call(&mut caller, (base20, len20 * 8, 4))?;
              result20
            }),
            _ => return Err(invalid_variant("Option")),
          }, match load14 {
            0 => None,
            1 => Some({
              let ptr21 = load15;
              let len21 = load16;
              
              copy_slice(
              &mut caller,
              memory,
              func_canonical_abi_free,
              ptr21, len21, 1
              )?
              
            }),
            _ => return Err(invalid_variant("Option")),
          }))
        }
      }
      use witx_bindgen_wasmtime::rt::RawMem;
      use witx_bindgen_wasmtime::rt::invalid_variant;
      use core::convert::TryFrom;
      use witx_bindgen_wasmtime::rt::bad_int;
      use witx_bindgen_wasmtime::rt::copy_slice;
    }
    