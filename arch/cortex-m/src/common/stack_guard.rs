use core::ptr::{read_volatile, write_volatile};

use specs::arch::{
    StackGuard,
    StackGuardContext,
    StackGuardError,
    StackGuardMode,
    StackGuardState,
};

pub struct CortexMStackGuard;

impl CortexMStackGuard {
    
    /// DESCRIPTION
    /// validate stack bounds before guard init/check.
    #[inline]
    fn validate_bounds(ctx: &StackGuardContext) -> Result<(), StackGuardError> {
        if ctx.stack_limit.is_null() || ctx.stack_top.is_null() {
            return Err(StackGuardError::InvalidStackBounds);
        }
        if (ctx.stack_top as usize) <= (ctx.stack_limit as usize) {
            return Err(StackGuardError::InvalidStackBounds);
        }
        Ok(())
    }

    /// DESCRIPTION
    /// get canary address at stack limit boundary.
    #[inline]
    fn canary_ptr(ctx: &StackGuardContext) -> *mut u32 {
        ctx.stack_limit.cast::<u32>()
    }
}

impl StackGuard for CortexMStackGuard {
    
    /// DESCRIPTION
    /// initialise stack guard metadata and seed canary/watermark state.
    fn initialise(&self, ctx: &mut StackGuardContext) -> Result<StackGuardState, StackGuardError> {
        Self::validate_bounds(ctx)?;

        match ctx.config.mode {
            StackGuardMode::Canary => {
                let p_canary = Self::canary_ptr(ctx);
                unsafe {
                    write_volatile(p_canary, ctx.config.canary_word);
                }
                ctx.state.low_mark = ctx.stack_limit;
            }
            StackGuardMode::Watermark => {
                ctx.state.low_mark = ctx.stack_top;
            }
        }

        Ok(ctx.state)
    }

    /// DESCRIPTION
    /// verify stack guard integrity for canary/watermark mode.
    fn check(&self, ctx: &mut StackGuardContext) -> Result<(), StackGuardError> {
        Self::validate_bounds(ctx)?;

        match ctx.config.mode {
            StackGuardMode::Canary => {
                let p_canary = Self::canary_ptr(ctx);
                let canary_read = unsafe { read_volatile(p_canary) };
                if canary_read != ctx.config.canary_word {
                    return Err(StackGuardError::GuardCorrupted);
                }
            }
            StackGuardMode::Watermark => {
                if (ctx.state.low_mark as usize) < (ctx.stack_limit as usize) {
                    return Err(StackGuardError::GuardCorrupted);
                }
            }
        }

        Ok(())
    }
}