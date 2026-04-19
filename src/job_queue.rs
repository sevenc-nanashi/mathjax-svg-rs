use boa_engine::context::time::JsInstant;
use boa_engine::job::{GenericJob, TimeoutJob};
use boa_engine::{
    Context, JsResult,
    job::{Job, JobExecutor, NativeAsyncJob, PromiseJob},
};
use futures_concurrency::future::FutureGroup;
use futures_lite::{StreamExt, future};
use std::{
    cell::RefCell,
    collections::{BTreeMap, VecDeque},
    mem,
    rc::Rc,
};

/// An event queue using pollster to drive futures to completion.
pub struct Queue {
    async_jobs: RefCell<VecDeque<NativeAsyncJob>>,
    promise_jobs: RefCell<VecDeque<PromiseJob>>,
    timeout_jobs: RefCell<BTreeMap<JsInstant, TimeoutJob>>,
    generic_jobs: RefCell<VecDeque<GenericJob>>,
}

impl Queue {
    pub fn new() -> Self {
        Self {
            async_jobs: RefCell::default(),
            promise_jobs: RefCell::default(),
            timeout_jobs: RefCell::default(),
            generic_jobs: RefCell::default(),
        }
    }

    fn clear(&self) {
        self.promise_jobs.borrow_mut().clear();
        self.async_jobs.borrow_mut().clear();
        self.timeout_jobs.borrow_mut().clear();
        self.generic_jobs.borrow_mut().clear();
    }
}

impl JobExecutor for Queue {
    fn enqueue_job(self: Rc<Self>, job: Job, context: &mut Context) {
        match job {
            Job::PromiseJob(job) => self.promise_jobs.borrow_mut().push_back(job),
            Job::AsyncJob(job) => self.async_jobs.borrow_mut().push_back(job),
            Job::TimeoutJob(t) => {
                let now = context.clock().now();
                self.timeout_jobs.borrow_mut().insert(now + t.timeout(), t);
            }
            Job::GenericJob(g) => self.generic_jobs.borrow_mut().push_back(g),
            _ => unreachable!("unsupported job type"),
        }
    }

    fn run_jobs(self: Rc<Self>, context: &mut Context) -> JsResult<()> {
        pollster::block_on(self.run_jobs_async(&RefCell::new(context)))
    }

    async fn run_jobs_async(self: Rc<Self>, context: &RefCell<&mut Context>) -> JsResult<()>
    where
        Self: Sized,
    {
        let mut group = FutureGroup::new();
        loop {
            for job in mem::take(&mut *self.async_jobs.borrow_mut()) {
                group.insert(job.call(context));
            }

            let no_timeout_jobs_to_run = {
                let now = context.borrow().clock().now();
                !self
                    .timeout_jobs
                    .borrow()
                    .iter()
                    .any(|(time, _)| &now >= time)
            };

            if self.promise_jobs.borrow().is_empty()
                && self.async_jobs.borrow().is_empty()
                && self.generic_jobs.borrow().is_empty()
                && no_timeout_jobs_to_run
                && group.is_empty()
            {
                break;
            }

            if let Some(Err(err)) = future::poll_once(group.next()).await.flatten() {
                self.clear();
                return Err(err);
            };

            {
                let now = context.borrow().clock().now();
                let mut timeouts_borrow = self.timeout_jobs.borrow_mut();
                let mut jobs_to_keep = timeouts_borrow.split_off(&now);
                jobs_to_keep.retain(|_, job| !job.is_cancelled());
                let jobs_to_run = mem::replace(&mut *timeouts_borrow, jobs_to_keep);
                drop(timeouts_borrow);

                for job in jobs_to_run.into_values() {
                    if let Err(err) = job.call(&mut context.borrow_mut()) {
                        self.clear();
                        return Err(err);
                    }
                }
            }

            let jobs = mem::take(&mut *self.promise_jobs.borrow_mut());
            for job in jobs {
                if let Err(err) = job.call(&mut context.borrow_mut()) {
                    self.clear();
                    return Err(err);
                }
            }

            let jobs = mem::take(&mut *self.generic_jobs.borrow_mut());
            for job in jobs {
                if let Err(err) = job.call(&mut context.borrow_mut()) {
                    self.clear();
                    return Err(err);
                }
            }

            context.borrow_mut().clear_kept_objects();
            future::yield_now().await;
        }

        Ok(())
    }
}
