use winapi::shared::{sspi, winerror};
use winapi::um::{minschannel};
use std::mem;
use std::ptr;
use std::io;

use {INIT_REQUESTS, Inner, secbuf, secbuf_desc};
use cert_context::CertContext;
use context_buffer::ContextBuffer;

use schannel_cred::SchannelCred;

pub struct SecurityContext(sspi::CtxtHandle);

impl Drop for SecurityContext {
    fn drop(&mut self) {
        unsafe {
            sspi::DeleteSecurityContext(&mut self.0);
        }
    }
}

impl Inner<sspi::CtxtHandle> for SecurityContext {
    unsafe fn from_inner(inner: sspi::CtxtHandle) -> SecurityContext {
        SecurityContext(inner)
    }

    fn as_inner(&self) -> sspi::CtxtHandle {
        self.0
    }

    fn get_mut(&mut self) -> &mut sspi::CtxtHandle {
        &mut self.0
    }
}

impl SecurityContext {
    pub fn initialize(cred: &mut SchannelCred,
                      accept: bool,
                      domain: Option<&[u16]>)
                      -> io::Result<(SecurityContext, Option<ContextBuffer>)> {
        unsafe {
            let mut ctxt = mem::zeroed();

            if accept {
                // If we're performing an accept then we need to wait to call
                // `AcceptSecurityContext` until we've actually read some data.
                return Ok((SecurityContext(ctxt), None))
            }

            let domain = domain.map(|b| b.as_ptr() as *mut u16).unwrap_or(ptr::null_mut());

            let mut outbuf = [secbuf(sspi::SECBUFFER_EMPTY, None)];
            let mut outbuf_desc = secbuf_desc(&mut outbuf);

            let mut attributes = 0;

            match sspi::InitializeSecurityContextW(cred.get_mut(),
                                                   ptr::null_mut(),
                                                   domain,
                                                   INIT_REQUESTS,
                                                   0,
                                                   0,
                                                   ptr::null_mut(),
                                                   0,
                                                   &mut ctxt,
                                                   &mut outbuf_desc,
                                                   &mut attributes,
                                                   ptr::null_mut()) {
                winerror::SEC_I_CONTINUE_NEEDED => {
                    Ok((SecurityContext(ctxt), Some(ContextBuffer(outbuf[0]))))
                }
                err => {
                    Err(io::Error::from_raw_os_error(err as i32))
                }
            }
        }
    }

    pub fn stream_sizes(&mut self) -> io::Result<sspi::SecPkgContext_StreamSizes> {
        unsafe {
            let mut stream_sizes = mem::zeroed();
            let status = sspi::QueryContextAttributesW(&mut self.0,
                                                       sspi::SECPKG_ATTR_STREAM_SIZES,
                                                       &mut stream_sizes as *mut _ as *mut _);
            if status == winerror::SEC_E_OK {
                Ok(stream_sizes)
            } else {
                Err(io::Error::from_raw_os_error(status as i32))
            }
        }
    }

    pub fn remote_cert(&mut self) -> io::Result<CertContext> {
        unsafe {
            let mut cert_context = mem::zeroed();
            let status = sspi::QueryContextAttributesW(&mut self.0,
                                                       minschannel::SECPKG_ATTR_REMOTE_CERT_CONTEXT,
                                                       &mut cert_context as *mut _ as *mut _);
            if status == winerror::SEC_E_OK {
                Ok(CertContext::from_inner(cert_context))
            } else {
                Err(io::Error::from_raw_os_error(status as i32))
            }
        }
    }
}
