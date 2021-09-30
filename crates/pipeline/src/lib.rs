use anyhow::Error;
use async_trait::async_trait;
use std::{marker::PhantomData, sync::Arc};

pub trait Subscription<Event>: Sync + Send {
    fn on_event(&self, input: Event) -> Result<(), Error>;
}

#[async_trait]
pub trait Binding: Clone + Send + Sync + 'static {
    type DataType: Send + Sync + 'static;
    async fn propagate_result(&self, input: Self::DataType) -> Result<(), Error>;
}

pub trait EventSource {
    type Event: Send + Sync + 'static;
}

#[async_trait]
pub trait Trigger: EventSource + Sized {
    async fn run<S: Subscription<Self::Event>>(&self, subscription: S) -> anyhow::Result<()>;
}

#[async_trait]
pub trait Consumer<Event: Send + Sync>: Clone + Send + Sync + 'static {
    async fn consume(&self, input: Event) -> Result<(), Error>;
}

pub struct Broker<Event: Send + Sync, S: Consumer<Event>> {
    pub subscriber: Arc<S>,
    pub _phantom: PhantomData<Event>,
}

impl<E: Send + Sync + 'static, C: Consumer<E>> Subscription<E> for Broker<E, C> {
    fn on_event(&self, event_data: E) -> Result<(), Error> {
        let subscriber = self.subscriber.clone();
        tokio::spawn(async move { subscriber.consume(event_data).await });
        Ok(())
    }
}

pub trait Converter<T, R>: Send + Sync + Clone + 'static + (Fn(T) -> anyhow::Result<R>) {}
impl<T, R, F: Send + Sync + Clone + 'static + (Fn(T) -> anyhow::Result<R>)> Converter<T, R> for F {}

pub struct ProcessingPipeline<
    Event: Send + Sync + 'static,
    OutputBinding: Binding,
    Engine: glass_engine::Executor,
    InputConverter: Converter<Event, Engine::Input>,
    OutputConverter: Converter<Engine::Output, OutputBinding::DataType>,
> {
    pub engine: Engine,
    pub binding: OutputBinding,
    pub input_converter: InputConverter,
    pub output_converter: OutputConverter,

    _phantom: PhantomData<Event>,
}

impl<
        Event: Send + Sync + 'static,
        OutputBinding: Binding,
        Engine: glass_engine::Executor,
        InputConverter: Converter<Event, Engine::Input>,
        OutputConverter: Converter<Engine::Output, OutputBinding::DataType>,
    > Clone for ProcessingPipeline<Event, OutputBinding, Engine, InputConverter, OutputConverter>
{
    fn clone(&self) -> Self {
        Self {
            engine: self.engine.clone(),
            binding: self.binding.clone(),
            input_converter: self.input_converter.clone(),
            output_converter: self.output_converter.clone(),
            _phantom: self._phantom.clone(),
        }
    }
}

#[async_trait]
impl<
        Event: Send + Sync + 'static,
        OutputBinding: Binding,
        Engine: glass_engine::Executor,
        InputConverter: Converter<Event, Engine::Input>,
        OutputConverter: Converter<Engine::Output, OutputBinding::DataType>,
    > Consumer<Event>
    for ProcessingPipeline<Event, OutputBinding, Engine, InputConverter, OutputConverter>
{
    async fn consume(&self, input: Event) -> Result<(), Error> {
        let converted_input = (self.input_converter)(input)?;
        let engine_output = self.engine.execute(converted_input).await?;
        let result = (self.output_converter)(engine_output)?;
        self.binding.propagate_result(result).await
    }
}

pub struct Pipeline<
    TriggerType: Trigger + Send + Sync + 'static,
    OutputBinding: Binding,
    Engine: glass_engine::Executor,
    InputConverter: Converter<TriggerType::Event, Engine::Input>,
    OutputConverter: Converter<Engine::Output, OutputBinding::DataType>,
> {
    pub trigger: TriggerType,
    pub consumer: Arc<
        ProcessingPipeline<
            TriggerType::Event,
            OutputBinding,
            Engine,
            InputConverter,
            OutputConverter,
        >,
    >,
}

impl<
        TriggerType: Trigger + Send + Sync + 'static,
        OutputBinding: Binding,
        Engine: glass_engine::Executor,
        InputConverter: Converter<TriggerType::Event, Engine::Input>,
        OutputConverter: Converter<Engine::Output, OutputBinding::DataType>,
    > Pipeline<TriggerType, OutputBinding, Engine, InputConverter, OutputConverter>
{
    pub async fn run(&self) -> anyhow::Result<()> {
        let broker = Broker {
            subscriber: self.consumer.clone(),
            _phantom: PhantomData,
        };
        self.trigger.run(broker).await
    }

    pub fn new(
        trigger: TriggerType,
        f_in: InputConverter,
        engine: Engine,
        f_out: OutputConverter,
        binding: OutputBinding,
    ) -> Self {
        Self {
            trigger,
            consumer: Arc::new(ProcessingPipeline {
                engine,
                input_converter: f_in,
                output_converter: f_out,
                binding,
                _phantom: PhantomData,
            }),
        }
    }
}
