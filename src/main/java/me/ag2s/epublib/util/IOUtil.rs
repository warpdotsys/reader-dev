use crate::me::ag2s::epublib::util::commons::io::IOConsumer;

/**
 * Most of the functions herein are re-implementations of the ones in
 * apache io IOUtils.
 * <p>
 * The reason for re-implementing this is that the functions are fairly simple
 * and using my own implementation saves the inclusion of a 200Kb jar file.
 */
pub struct IOUtil;

impl IOUtil {
    const TAG: &'static str = "me.ag2s.epublib.util.IOUtil";

    /**
     * Represents the end-of-file (or stream).
     *
     * @since 2.5 (made public)
     */
    pub const EOF: i32 = -1;

    pub const DEFAULT_BUFFER_SIZE: usize = 1024 * 8;
    const SKIP_BYTE_BUFFER: [u8; 1024 * 8] = [0; 1024 * 8];

    // Allocated in the relevant skip method if necessary.
    /*
     * These buffers are static and are shared between threads.
     * This is possible because the buffers are write-only - the contents are never read.
     *
     * N.B. there is no need to synchronize when creating these because:
     * - we don't care if the buffer is created multiple times (the data is ignored)
     * - we always use the same size buffer, so if it it is recreated it will still be OK
     * (if the buffer size were variable, we would need to synch. to ensure some other thread
     * did not create a smaller one)
     */
    static mut SKIP_CHAR_BUFFER: Option<Vec<char>> = None;

    /**
     * Gets the contents of the Reader as a byte[], with the given character encoding.
     *
     * @param in       g
     * @param encoding g
     * @return the contents of the Reader as a byte[], with the given character encoding.
     * @throws IOException g
     */
    pub fn to_byte_array_reader(in_reader: &Reader, encoding: &str) -> Result<Vec<u8>, io::Error> {
        let mut out = StringWriter::new();
        copy_reader_app(out)?;
        out.flush();
        Ok(out.to_string().into_bytes())
    }

    /**
     * Returns the contents of the InputStream as a byte[]
     *
     * @param in f
     * @return the contents of the InputStream as a byte[]
     * @throws IOException f
     */
    pub fn to_byte_array(in_stream: &InputStream) -> Result<Vec<u8>, io::Error> {
        let mut result = ByteArrayOutputStream::new();
        copy(in_stream, &mut result)?;
        result.flush();
        Ok(result.to_byte_array())
    }

    /**
     * Reads data from the InputStream, using the specified buffer size.
     * <p>
     * This is meant for situations where memory is tight, since
     * it prevents buffer expansion.
     *
     * @param in   the stream to read data from
     * @param size the size of the array to create
     * @return the array, or null
     * @throws IOException f
     */
    pub fn to_byte_array_size(in_stream: &InputStream, size: usize) -> Result<Option<Vec<u8>>, io::Error> {

        let mut result;

        if size > 0 {
            result = ByteArrayOutputStream::new_size(size);
        } else {
            result = ByteArrayOutputStream::new();
        }

        copy(in_stream, &mut result)?;
        result.flush();
        Ok(Some(result.to_byte_array()))
    }

    /**
     * if totalNrRead &lt; 0 then totalNrRead is returned, if
     * (nrRead + totalNrRead) &lt; Integer.MAX_VALUE then nrRead + totalNrRead
     * is returned, -1 otherwise.
     *
     * @param nrRead       f
     * @param totalNrNread f
     * @return if totalNrRead &lt; 0 then totalNrRead is returned, if
     * (nrRead + totalNrRead) &lt; Integer.MAX_VALUE then nrRead + totalNrRead
     * is returned, -1 otherwise.
     */
    fn calc_new_nr_read_size(nr_read: i32, total_nr_nread: i32) -> i32 {
        if total_nr_nread < 0 {
            return total_nr_nread;
        }
        if total_nr_nread > (i32::MAX - nr_read) {
            return -1;
        } else {
            return (total_nr_nread + nr_read);
        }
    }

    //
    pub fn copy(in_stream: &InputStream, result: &mut OutputStream) -> Result<(), io::Error> {
        copy_buffer(in_stream, result, IOUtil::DEFAULT_BUFFER_SIZE)
    }

    /**
     * Copies bytes from an <code>InputStream</code> to an <code>OutputStream</code> using an internal buffer of the
     * given size.
     * <p>
     * This method buffers the input internally, so there is no need to use a <code>BufferedInputStream</code>.
     * </p>
     *
     * @param input      the <code>InputStream</code> to read from
     * @param output     the <code>OutputStream</code> to write to
     * @param bufferSize the bufferSize used to copy from the input to the output
     * @return the number of bytes copied. or {@code 0} if {@code input is null}.
     * @throws NullPointerException if the output is null
     * @throws IOException          if an I/O error occurs
     * @since 2.5
     */
    pub fn copy_buffer(input: &InputStream, output: &mut OutputStream, buffer_size: usize) -> Result<(), io::Error> {
        copy_large(input, output, vec![0u8; buffer_size]);
        Ok(())
    }

    /**
     * Copies bytes from an <code>InputStream</code> to chars on a
     * <code>Writer</code> using the default character encoding of the platform.
     * <p>
     * This method buffers the input internally, so there is no need to use a
     * <code>BufferedInputStream</code>.
     * <p>
     * This method uses {@link InputStreamReader}.
     *
     * @param input  the <code>InputStream</code> to read from
     * @param output the <code>Writer</code> to write to
     * @throws NullPointerException if the input or output is null
     * @throws IOException          if an I/O error occurs
     * @since 1.1
     * @deprecated 2.5 use {@link #copy(InputStream, Writer, Charset)} instead
     */
    #[deprecated]
    pub fn copy_input_writer(input: &InputStream, output: &mut Writer) -> Result<(), io::Error> {
        copy_input_writer_charset(input, output, Charset::default_charset())
    }

    /**
     * Copies bytes from an <code>InputStream</code> to chars on a
     * <code>Writer</code> using the specified character encoding.
     * <p>
     * This method buffers the input internally, so there is no need to use a
     * <code>BufferedInputStream</code>.
     * <p>
     * This method uses {@link InputStreamReader}.
     *
     * @param input        the <code>InputStream</code> to read from
     * @param output       the <code>Writer</code> to write to
     * @param inputCharset the charset to use for the input stream, null means platform default
     * @throws NullPointerException if the input or output is null
     * @throws IOException          if an I/O error occurs
     * @since 2.3
     */
    pub fn copy_input_writer_charset(input: &InputStream, output: &mut Writer, input_charset: Charset) -> Result<(), io::Error> {
        let in_reader = InputStreamReader::new(input, input_charset.name());
        copy_reader_writer(&in_reader, output);
        Ok(())
    }

    /**
     * Copies bytes from an <code>InputStream</code> to chars on a
     * <code>Writer</code> using the specified character encoding.
     * <p>
     * This method buffers the input internally, so there is no need to use a
     * <code>BufferedInputStream</code>.
     * <p>
     * Character encoding names can be found at
     * <a href="http://www.iana.org/assignments/character-sets">IANA</a>.
     * <p>
     * This method uses {@link InputStreamReader}.
     *
     * @param input            the <code>InputStream</code> to read from
     * @param output           the <code>Writer</code> to write to
     * @param inputCharsetName the name of the requested charset for the InputStream, null means platform default
     * @throws NullPointerException                         if the input or output is null
     * @throws IOException                                  if an I/O error occurs
     * @throws java.nio.charset.UnsupportedCharsetException thrown instead of {@link java.io
     *                                                      .UnsupportedEncodingException} in version 2.2 if the
     *                                                      encoding is not supported.
     * @since 1.1
     */
    pub fn copy_input_writer_charset_name(input: &InputStream, output: &mut Writer, input_charset_name: &str) -> Result<(), io::Error> {
        copy_input_writer_charset(input, output, Charset::for_name(input_charset_name))
    }

    /**
     * Copies chars from a <code>Reader</code> to a <code>Appendable</code>.
     * <p>
     * This method buffers the input internally, so there is no need to use a
     * <code>BufferedReader</code>.
     * <p>
     * Large streams (over 2GB) will return a chars copied value of
     * <code>-1</code> after the copy has completed since the correct
     * number of chars cannot be returned as an int. For large streams
     * use the <code>copyLarge(Reader, Writer)</code> method.
     *
     * @param input  the <code>Reader</code> to read from
     * @param output the <code>Appendable</code> to write to
     * @return the number of characters copied, or -1 if &gt; Integer.MAX_VALUE
     * @throws NullPointerException if the input or output is null
     * @throws IOException          if an I/O error occurs
     * @since 2.7
     */
    pub fn copy_reader_app(input: &Reader, output: &mut Appendable) -> Result<i64, io::Error> {
        copy_reader_app_buffer(input, output, CharBuffer::allocate(IOUtil::DEFAULT_BUFFER_SIZE))
    }

    /**
     * Copies chars from a <code>Reader</code> to an <code>Appendable</code>.
     * <p>
     * This method uses the provided buffer, so there is no need to use a
     * <code>BufferedReader</code>.
     * </p>
     *
     * @param input  the <code>Reader</code> to read from
     * @param output the <code>Appendable</code> to write to
     * @param buffer the buffer to be used for the copy
     * @return the number of characters copied
     * @throws NullPointerException if the input or output is null
     * @throws IOException          if an I/O error occurs
     * @since 2.7
     */
    pub fn copy_reader_app_buffer(input: &Reader, output: &mut Appendable, buffer: CharBuffer) -> Result<i64, io::Error> {
        let mut count = 0;
        loop {
            let n = input.read(&mut buffer);
            if IOUtil::EOF == n {
                break;
            }
            buffer.flip();
            output.append(&buffer, 0, n);
            count += n;
        }
        Ok(count)
    }

    /**
     * Copies chars from a <code>Reader</code> to bytes on an
     * <code>OutputStream</code> using the default character encoding of the
     * platform, and calling flush.
     * <p>
     * This method buffers the input internally, so there is no need to use a
     * <code>BufferedReader</code>.
     * <p>
     * Due to the implementation of OutputStreamWriter, this method performs a
     * flush.
     * <p>
     * This method uses {@link OutputStreamWriter}.
     *
     * @param input  the <code>Reader</code> to read from
     * @param output the <code>OutputStream</code> to write to
     * @throws NullPointerException if the input or output is null
     * @throws IOException          if an I/O error occurs
     * @since 1.1
     * @deprecated 2.5 use {@link #copy(Reader, OutputStream, Charset)} instead
     */
    #[deprecated]
    pub fn copy_reader_output(input: &Reader, output: &mut OutputStream) -> Result<(), io::Error> {
        copy_reader_output_charset(input, output, Charset::default_charset())
    }

    /**
     * Copies chars from a <code>Reader</code> to bytes on an
     * <code>OutputStream</code> using the specified character encoding, and
     * calling flush.
     * <p>
     * This method buffers the input internally, so there is no need to use a
     * <code>BufferedReader</code>.
     * </p>
     * <p>
     * Due to the implementation of OutputStreamWriter, this method performs a
     * flush.
     * </p>
     * <p>
     * This method uses {@link OutputStreamWriter}.
     * </p>
     *
     * @param input         the <code>Reader</code> to read from
     * @param output        the <code>OutputStream</code> to write to
     * @param outputCharset the charset to use for the OutputStream, null means platform default
     * @throws NullPointerException if the input or output is null
     * @throws IOException          if an I/O error occurs
     * @since 2.3
     */
    pub fn copy_reader_output_charset(input: &Reader, output: &mut OutputStream, output_charset: Charset) -> Result<(), io::Error> {
        let mut out = OutputStreamWriter::new(output, output_charset.name());
        copy_reader_writer(input, &mut out)?;
        // XXX Unless anyone is planning on rewriting OutputStreamWriter,
        // we have to flush here.
        out.flush();
        Ok(())
    }

    /**
     * Copies chars from a <code>Reader</code> to bytes on an
     * <code>OutputStream</code> using the specified character encoding, and
     * calling flush.
     * <p>
     * This method buffers the input internally, so there is no need to use a
     * <code>BufferedReader</code>.
     * <p>
     * Character encoding names can be found at
     * <a href="http://www.iana.org/assignments/character-sets">IANA</a>.
     * <p>
     * Due to the implementation of OutputStreamWriter, this method performs a
     * flush.
     * <p>
     * This method uses {@link OutputStreamWriter}.
     *
     * @param input             the <code>Reader</code> to read from
     * @param output            the <code>OutputStream</code> to write to
     * @param outputCharsetName the name of the requested charset for the OutputStream, null means platform default
     * @throws NullPointerException                         if the input or output is null
     * @throws IOException                                  if an I/O error occurs
     * @throws java.nio.charset.UnsupportedCharsetException thrown instead of {@link java.io
     *                                                      .UnsupportedEncodingException} in version 2.2 if the
     *                                                      encoding is not supported.
     * @since 1.1
     */
    pub fn copy_reader_output_charset_name(input: &Reader, output: &mut OutputStream, output_charset_name: &str) -> Result<(), io::Error> {
        copy_reader_output_charset(input, output, Charset::for_name(output_charset_name))
    }

    /**
     * Copies chars from a <code>Reader</code> to a <code>Writer</code>.
     * <p>
     * This method buffers the input internally, so there is no need to use a
     * <code>BufferedReader</code>.
     * <p>
     * Large streams (over 2GB) will return a chars copied value of
     * <code>-1</code> after the copy has completed since the correct
     * number of chars cannot be returned as an int. For large streams
     * use the <code>copyLarge(Reader, Writer)</code> method.
     *
     * @param input  the <code>Reader</code> to read from
     * @param output the <code>Writer</code> to write to
     * @return the number of characters copied, or -1 if &gt; Integer.MAX_VALUE
     * @throws NullPointerException if the input or output is null
     * @throws IOException          if an I/O error occurs
     * @since 1.1
     */
    pub fn copy_reader_writer(input: &Reader, output: &mut Writer) -> Result<i32, io::Error> {
        let count = copy_large_reader(input, output);
        if count > i32::MAX as i64 {
            return Ok(-1);
        }
        Ok(count as i32)
    }

    /**
     * Copies bytes from a large (over 2GB) <code>InputStream</code> to an
     * <code>OutputStream</code>.
     * <p>
     * This method buffers the input internally, so there is no need to use a
     * <code>BufferedInputStream</code>.
     * </p>
     * <p>
     * The buffer size is given by {@link #DEFAULT_BUFFER_SIZE}.
     * </p>
     *
     * @param input  the <code>InputStream</code> to read from
     * @param output the <code>OutputStream</code> to write to
     * @return the number of bytes copied. or {@code 0} if {@code input is null}.
     * @throws NullPointerException if the output is null
     * @throws IOException          if an I/O error occurs
     * @since 1.3
     */
    pub fn copy_large(input: &InputStream, output: &mut OutputStream) -> Result<i64, io::Error> {
        copy_buffer(input, output, IOUtil::DEFAULT_BUFFER_SIZE)
    }

    /**
     * Copies bytes from a large (over 2GB) <code>InputStream</code> to an
     * <code>OutputStream</code>.
     * <p>
     * This method uses the provided buffer, so there is no need to use a
     * <code>BufferedInputStream</code>.
     * </p>
     *
     * @param input  the <code>InputStream</code> to read from
     * @param output the <code>OutputStream</code> to write to
     * @param buffer the buffer to use for the copy
     * @return the number of bytes copied. or {@code 0} if {@code input is null}.
     * @throws IOException if an I/O error occurs
     * @since 2.2
     */
    pub fn copy_large_buffer(input: &InputStream, output: &mut OutputStream, buffer: &mut [u8]) -> Result<i64, io::Error> {
        let mut count = 0;
        loop {
            let n = input.read(buffer);
            if IOUtil::EOF == n {
                break;
            }
            output.write(buffer, 0, n);
            count += n;
        }
        Ok(count)
    }

    /**
     * Copies some or all bytes from a large (over 2GB) <code>InputStream</code> to an
     * <code>OutputStream</code>, optionally skipping input bytes.
     * <p>
     * This method buffers the input internally, so there is no need to use a
     * <code>BufferedInputStream</code>.
     * </p>
     * <p>
     * Note that the implementation uses {@link #skip(InputStream, long)}.
     * This means that the method may be considerably less efficient than using the actual skip implementation,
     * this is done to guarantee that the correct number of characters are skipped.
     * </p>
     * The buffer size is given by {@link #DEFAULT_BUFFER_SIZE}.
     *
     * @param input       the <code>InputStream</code> to read from
     * @param output      the <code>OutputStream</code> to write to
     * @param inputOffset : number of bytes to skip from input before copying
     *                    -ve values are ignored
     * @param length      : number of bytes to copy. -ve means all
     * @return the number of bytes copied
     * @throws NullPointerException if the input or output is null
     * @throws IOException          if an I/O error occurs
     * @since 2.2
     */
    pub fn copy_large_offset(input: &InputStream, output: &mut OutputStream, input_offset: i64, length: i64) -> Result<i64, io::Error> {
        copy_large_offset_buffer(input, output, input_offset, length, vec![0u8; IOUtil::DEFAULT_BUFFER_SIZE])
    }

    /**
     * Copies some or all bytes from a large (over 2GB) <code>InputStream</code> to an
     * <code>OutputStream</code>, optionally skipping input bytes.
     * <p>
     * This method uses the provided buffer, so there is no need to use a
     * <code>BufferedInputStream</code>.
     * </p>
     * <p>
     * Note that the implementation uses {@link #skip(InputStream, long)}.
     * This means that the method may be considerably less efficient than using the actual skip implementation,
     * this is done to guarantee that the correct number of characters are skipped.
     * </p>
     *
     * @param input       the <code>InputStream</code> to read from
     * @param output      the <code>OutputStream</code> to write to
     * @param inputOffset : number of bytes to skip from input before copying
     *                    -ve values are ignored
     * @param length      : number of bytes to copy. -ve means all
     * @param buffer      the buffer to use for the copy
     * @return the number of bytes copied
     * @throws NullPointerException if the input or output is null
     * @throws IOException          if an I/O error occurs
     * @since 2.2
     */
    pub fn copy_large_offset_buffer(input: &InputStream, output: &mut OutputStream,
                                    input_offset: i64, length: i64, buffer: Vec<u8>) -> Result<i64, io::Error> {
        if input_offset > 0 {
            skip_fully(input, input_offset)?;
        }
        if length == 0 {
            return Ok(0);
        }
        let buffer_length = buffer.len();
        let mut bytes_to_read = buffer_length;
        if length > 0 && length < buffer_length as i64 {
            bytes_to_read = length as usize;
        }
        let mut total_read: i64 = 0;
        while bytes_to_read > 0 {
            let read = input.read(buffer, 0, bytes_to_read);
            if IOUtil::EOF == read {
                break;
            }
            output.write(buffer, 0, read);
            total_read += read as i64;
            if length > 0 { // only adjust length if not reading to the end
                // Note the cast must work because buffer.length is an integer
                bytes_to_read = std::cmp::min(length - total_read, buffer_length as i64) as usize;
            }
        }
        Ok(total_read)
    }

    /**
     * Copies chars from a large (over 2GB) <code>Reader</code> to a <code>Writer</code>.
     * <p>
     * This method buffers the input internally, so there is no need to use a
     * <code>BufferedReader</code>.
     * <p>
     * The buffer size is given by {@link #DEFAULT_BUFFER_SIZE}.
     *
     * @param input  the <code>Reader</code> to read from
     * @param output the <code>Writer</code> to write to
     * @return the number of characters copied
     * @throws NullPointerException if the input or output is null
     * @throws IOException          if an I/O error occurs
     * @since 1.3
     */
    pub fn copy_large_reader(input: &Reader, output: &mut Writer) -> Result<i64, io::Error> {
        copy_large_reader_buffer(input, output, vec!['\0'; IOUtil::DEFAULT_BUFFER_SIZE])
    }

    /**
     * Copies chars from a large (over 2GB) <code>Reader</code> to a <code>Writer</code>.
     * <p>
     * This method uses the provided buffer, so there is no need to use a
     * <code>BufferedReader</code>.
     * <p>
     *
     * @param input  the <code>Reader</code> to read from
     * @param output the <code>Writer</code> to write to
     * @param buffer the buffer to be used for the copy
     * @return the number of characters copied
     * @throws NullPointerException if the input or output is null
     * @throws IOException          if an I/O error occurs
     * @since 2.2
     */
    pub fn copy_large_reader_buffer(input: &Reader, output: &mut Writer, buffer: &mut [char]) -> Result<i64, io::Error> {
        let mut count = 0;
        loop {
            let n = input.read(buffer);
            if IOUtil::EOF == n {
                break;
            }
            output.write(buffer, 0, n);
            count += n as i64;
        }
        Ok(count)
    }

    /**
     * Copies some or all chars from a large (over 2GB) <code>InputStream</code> to an
     * <code>OutputStream</code>, optionally skipping input chars.
     * <p>
     * This method buffers the input internally, so there is no need to use a
     * <code>BufferedReader</code>.
     * <p>
     * The buffer size is given by {@link #DEFAULT_BUFFER_SIZE}.
     *
     * @param input       the <code>Reader</code> to read from
     * @param output      the <code>Writer</code> to write to
     * @param inputOffset : number of chars to skip from input before copying
     *                    -ve values are ignored
     * @param length      : number of chars to copy. -ve means all
     * @return the number of chars copied
     * @throws NullPointerException if the input or output is null
     * @throws IOException          if an I/O error occurs
     * @since 2.2
     */
    pub fn copy_large_reader_offset(input: &Reader, output: &mut Writer, input_offset: i64, length: i64) -> Result<i64, io::Error> {
        copy_large_reader_offset_buffer(input, output, input_offset, length, vec!['\0'; IOUtil::DEFAULT_BUFFER_SIZE])
    }

    /**
     * Copies some or all chars from a large (over 2GB) <code>InputStream</code> to an
     * <code>OutputStream</code>, optionally skipping input chars.
     * <p>
     * This method uses the provided buffer, so there is no need to use a
     * <code>BufferedReader</code>.
     * <p>
     *
     * @param input       the <code>Reader</code> to read from
     * @param output      the <code>Writer</code> to write to
     * @param inputOffset : number of chars to skip from input before copying
     *                    -ve values are ignored
     * @param length      : number of chars to copy. -ve means all
     * @param buffer      the buffer to be used for the copy
     * @return the number of chars copied
     * @throws NullPointerException if the input or output is null
     * @throws IOException          if an I/O error occurs
     * @since 2.2
     */
    pub fn copy_large_reader_offset_buffer(input: &Reader, output: &mut Writer, input_offset: i64, length: i64,
                                           buffer: &mut [char]) -> Result<i64, io::Error> {
        if input_offset > 0 {
            skip_fully_reader(input, input_offset)?;
        }
        if length == 0 {
            return Ok(0);
        }
        let mut bytes_to_read = buffer.len();
        if length > 0 && length < buffer.len() as i64 {
            bytes_to_read = length as usize;
        }
        let mut total_read: i64 = 0;
        while bytes_to_read > 0 {
            let read = input.read(buffer, 0, bytes_to_read);
            if IOUtil::EOF == read {
                break;
            }
            output.write(buffer, 0, read);
            total_read += read as i64;
            if length > 0 { // only adjust length if not reading to the end
                // Note the cast must work because buffer.length is an integer
                bytes_to_read = std::cmp::min(length - total_read, buffer.len() as i64) as usize;
            }
        }
        Ok(total_read)
    }

    /**
     * Skips bytes from an input byte stream.
     * This implementation guarantees that it will read as many bytes
     * as possible before giving up; this may not always be the case for
     * skip() implementations in subclasses of {@link InputStream}.
     * <p>
     * Note that the implementation uses {@link InputStream#read(byte[], int, int)} rather
     * than delegating to {@link InputStream#skip(long)}.
     * This means that the method may be considerably less efficient than using the actual skip implementation,
     * this is done to guarantee that the correct number of bytes are skipped.
     * </p>
     *
     * @param input  byte stream to skip
     * @param toSkip number of bytes to skip.
     * @return number of bytes actually skipped.
     * @throws IOException              if there is a problem reading the file
     * @throws IllegalArgumentException if toSkip is negative
     * @see InputStream#skip(long)
     * @see <a href="https://issues.apache.org/jira/browse/IO-203">IO-203 - Add skipFully() method for InputStreams</a>
     * @since 2.0
     */
    pub fn skip(input: &InputStream, to_skip: i64) -> Result<i64, io::Error> {
        if to_skip < 0 {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "Skip count must be non-negative, actual: ".to_string() + &to_skip.to_string()));
        }
        /*
         * N.B. no need to synchronize access to SKIP_BYTE_BUFFER: - we don't care if the buffer is created multiple
         * times (the data is ignored) - we always use the same size buffer, so if it it is recreated it will still be
         * OK (if the buffer size were variable, we would need to synch. to ensure some other thread did not create a
         * smaller one)
         */
        let mut remain = to_skip;
        let mut skip_byte_buffer = IOUtil::SKIP_BYTE_BUFFER;
        while remain > 0 {
            // See https://issues.apache.org/jira/browse/IO-203 for why we use read() rather than delegating to skip()
            let n = input.read(&mut skip_byte_buffer, 0, std::cmp::min(remain, IOUtil::SKIP_BYTE_BUFFER.len() as i64) as usize);
            if n < 0 { // EOF
                break;
            }
            remain -= n as i64;
        }
        Ok(to_skip - remain)
    }

    /**
     * Skips bytes from a ReadableByteChannel.
     * This implementation guarantees that it will read as many bytes
     * as possible before giving up.
     *
     * @param input  ReadableByteChannel to skip
     * @param toSkip number of bytes to skip.
     * @return number of bytes actually skipped.
     * @throws IOException              if there is a problem reading the ReadableByteChannel
     * @throws IllegalArgumentException if toSkip is negative
     * @since 2.5
     */
    pub fn skip_channel(input: &mut ReadableByteChannel, to_skip: i64) -> Result<i64, io::Error> {
        if to_skip < 0 {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "Skip count must be non-negative, actual: ".to_string() + &to_skip.to_string()));
        }
        let mut skip_byte_buffer = ByteBuffer::allocate(std::cmp::min(to_skip, IOUtil::SKIP_BYTE_BUFFER.len() as i64) as usize);
        let mut remain = to_skip;
        while remain > 0 {
            skip_byte_buffer.position(0);
            skip_byte_buffer.limit(std::cmp::min(remain, IOUtil::SKIP_BYTE_BUFFER.len() as i64) as usize);
            let n = input.read(&mut skip_byte_buffer);
            if n == IOUtil::EOF {
                break;
            }
            remain -= n as i64;
        }
        Ok(to_skip - remain)
    }

    /**
     * Skips characters from an input character stream.
     * This implementation guarantees that it will read as many characters
     * as possible before giving up; this may not always be the case for
     * skip() implementations in subclasses of {@link Reader}.
     * <p>
     * Note that the implementation uses {@link Reader#read(char[], int, int)} rather
     * than delegating to {@link Reader#skip(long)}.
     * This means that the method may be considerably less efficient than using the actual skip implementation,
     * this is done to guarantee that the correct number of characters are skipped.
     * </p>
     *
     * @param input  character stream to skip
     * @param toSkip number of characters to skip.
     * @return number of characters actually skipped.
     * @throws IOException              if there is a problem reading the file
     * @throws IllegalArgumentException if toSkip is negative
     * @see Reader#skip(long)
     * @see <a href="https://issues.apache.org/jira/browse/IO-203">IO-203 - Add skipFully() method for InputStreams</a>
     * @since 2.0
     */
    pub fn skip_reader(input: &Reader, to_skip: i64) -> Result<i64, io::Error> {
        if to_skip < 0 {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "Skip count must be non-negative, actual: ".to_string() + &to_skip.to_string()));
        }
        /*
         * N.B. no need to synchronize this because: - we don't care if the buffer is created multiple times (the data
         * is ignored) - we always use the same size buffer, so if it it is recreated it will still be OK (if the buffer
         * size were variable, we would need to synch. to ensure some other thread did not create a smaller one)
         */
        if IOUtil::SKIP_CHAR_BUFFER == null {
            IOUtil::SKIP_CHAR_BUFFER = vec!['\0'; IOUtil::SKIP_BYTE_BUFFER.len()];
        }
        let mut remain = to_skip;
        let mut skip_char_buffer = IOUtil::SKIP_CHAR_BUFFER.clone();
        while remain > 0 {
            // See https://issues.apache.org/jira/browse/IO-203 for why we use read() rather than delegating to skip()
            let n = input.read(&mut skip_char_buffer, 0, std::cmp::min(remain, IOUtil::SKIP_BYTE_BUFFER.len() as i64) as usize);
            if n < 0 { // EOF
                break;
            }
            remain -= n as i64;
        }
        Ok(to_skip - remain)
    }

    /**
     * Skips the requested number of bytes or fail if there are not enough left.
     * <p>
     * This allows for the possibility that {@link InputStream#skip(long)} may
     * not skip as many bytes as requested (most likely because of reaching EOF).
     * <p>
     * Note that the implementation uses {@link #skip(InputStream, long)}.
     * This means that the method may be considerably less efficient than using the actual skip implementation,
     * this is done to guarantee that the correct number of characters are skipped.
     * </p>
     *
     * @param input  stream to skip
     * @param toSkip the number of bytes to skip
     * @throws IOException              if there is a problem reading the file
     * @throws IllegalArgumentException if toSkip is negative
     * @throws EOFException             if the number of bytes skipped was incorrect
     * @see InputStream#skip(long)
     * @since 2.0
     */
    pub fn skip_fully(input: &InputStream, to_skip: i64) -> Result<(), io::Error> {
        if to_skip < 0 {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "Bytes to skip must not be negative: ".to_string() + &to_skip.to_string()));
        }
        let skipped = skip(input, to_skip)?;
        if skipped != to_skip {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "Bytes to skip: ".to_string() + &to_skip.to_string() + " actual: " + &skipped.to_string()));
        }
        Ok(())
    }

    /**
     * Skips the requested number of bytes or fail if there are not enough left.
     *
     * @param input  ReadableByteChannel to skip
     * @param toSkip the number of bytes to skip
     * @throws IOException              if there is a problem reading the ReadableByteChannel
     * @throws IllegalArgumentException if toSkip is negative
     * @throws EOFException             if the number of bytes skipped was incorrect
     * @since 2.5
     */
    pub fn skip_fully_channel(input: &mut ReadableByteChannel, to_skip: i64) -> Result<(), io::Error> {
        if to_skip < 0 {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "Bytes to skip must not be negative: ".to_string() + &to_skip.to_string()));
        }
        let skipped = skip_channel(input, to_skip)?;
        if skipped != to_skip {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "Bytes to skip: ".to_string() + &to_skip.to_string() + " actual: " + &skipped.to_string()));
        }
        Ok(())
    }

    /**
     * Skips the requested number of characters or fail if there are not enough left.
     * <p>
     * This allows for the possibility that {@link Reader#skip(long)} may
     * not skip as many characters as requested (most likely because of reaching EOF).
     * <p>
     * Note that the implementation uses {@link #skip(Reader, long)}.
     * This means that the method may be considerably less efficient than using the actual skip implementation,
     * this is done to guarantee that the correct number of characters are skipped.
     * </p>
     *
     * @param input  stream to skip
     * @param toSkip the number of characters to skip
     * @throws IOException              if there is a problem reading the file
     * @throws IllegalArgumentException if toSkip is negative
     * @throws EOFException             if the number of characters skipped was incorrect
     * @see Reader#skip(long)
     * @since 2.0
     */
    pub fn skip_fully_reader(input: &Reader, to_skip: i64) -> Result<(), io::Error> {
        let skipped = skip_reader(input, to_skip)?;
        if skipped != to_skip {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "Chars to skip: ".to_string() + &to_skip.to_string() + " actual: " + &skipped.to_string()));
        }
        Ok(())
    }

    /**
     * Returns the length of the given array in a null-safe manner.
     *
     * @param array an array or null
     * @return the array length -- or 0 if the given array is null.
     * @since 2.7
     */
    pub fn length_byte(array: &[u8]) -> usize {
        array.len()
    }

    /**
     * Returns the length of the given array in a null-safe manner.
     *
     * @param array an array or null
     * @return the array length -- or 0 if the given array is null.
     * @since 2.7
     */
    pub fn length_char(array: &[char]) -> usize {
        array.len()
    }

    /**
     * Returns the length of the given CharSequence in a null-safe manner.
     *
     * @param csq a CharSequence or null
     * @return the CharSequence length -- or 0 if the given CharSequence is null.
     * @since 2.7
     */
    pub fn length_str(csq: &str) -> usize {
        csq.chars().count()
    }

    /**
     * Returns the length of the given array in a null-safe manner.
     *
     * @param array an array or null
     * @return the array length -- or 0 if the given array is null.
     * @since 2.7
     */
    pub fn length_obj(array: &[Object]) -> usize {
        array.len()
    }

    /**
     * Closes the given {@link Closeable} as a null-safe operation.
     *
     * @param closeable The resource to close, may be null.
     * @throws IOException if an I/O error occurs.
     * @since 2.7
     */
    pub fn close(closeable: &mut dyn Closeable) -> Result<(), io::Error> {
        if closeable != null {
            closeable.close();
        }
        Ok(())
    }

    /**
     * Closes the given {@link Closeable} as a null-safe operation.
     *
     * @param closeables The resource(s) to close, may be null.
     * @throws IOException if an I/O error occurs.
     * @since 2.8.0
     */
    pub fn close_all(closeables: Vec<&mut dyn Closeable>) -> Result<(), io::Error> {
        for closeable in closeables {
            close(closeable)?;
        }
        Ok(())
    }

    /**
     * Closes the given {@link Closeable} as a null-safe operation.
     *
     * @param closeable The resource to close, may be null.
     * @param consumer  Consume the IOException thrown by {@link Closeable#close()}.
     * @throws IOException if an I/O error occurs.
     * @since 2.7
     */
    pub fn close_consumer(closeable: &mut dyn Closeable, consumer: &mut dyn IOConsumer) -> Result<(), io::Error> {
        if closeable != null {
            match closeable.close() {
                Ok(_) => {}
                Err(e) => {
                    if consumer != null {
                        consumer.accept(e);
                    }
                }
            }
        }
        Ok(())
    }

    /**
     * Closes a URLConnection.
     *
     * @param conn the connection to close.
     * @since 2.4
     */
    pub fn close_url_connection(conn: &URLConnection) {
        if conn.is_http_url_connection() {
            conn.disconnect();
        }
    }

    #[allow(dead_code)]
    pub fn stream_2_string(input_stream: &InputStream) -> String {
        let mut result = ByteArrayOutputStream::new();
        match {
            let mut buffer = vec![0u8; IOUtil::DEFAULT_BUFFER_SIZE];
            loop {
                let length = input_stream.read(&mut buffer);
                if length == -1 {
                    break;
                }
                result.write(buffer, 0, length);
            }
            Ok(())
        } {
            Ok(_) => result.to_string(),
            Err(e) => e.to_string(),
        }
    }
}

pub struct Reader;
pub struct Writer;
pub struct StringWriter;
pub struct ByteArrayOutputStream;
pub struct InputStream;
pub struct OutputStream;
pub struct InputStreamReader;
pub struct OutputStreamWriter;
pub struct Appendable;
pub struct CharBuffer;
pub struct Charset;
pub struct ReadableByteChannel;
pub struct ByteBuffer;
pub struct Object;
pub struct Closeable;
pub struct URLConnection;
pub struct HttpURLConnection;

impl StringWriter {
    pub fn new() -> Self { todo!() }
    pub fn flush(&mut self) { todo!() }
    pub fn to_string(&self) -> String { todo!() }
}

impl ByteArrayOutputStream {
    pub fn new() -> Self { todo!() }
    pub fn new_size(_size: usize) -> Self { todo!() }
    pub fn flush(&mut self) { todo!() }
    pub fn to_byte_array(&self) -> Vec<u8> { todo!() }
    pub fn write(&mut self, _buffer: &[u8], _off: usize, _len: usize) { todo!() }
    pub fn to_string(&self) -> String { todo!() }
}

impl OutputStream {
    pub fn write(&mut self, _buffer: &[u8], _off: usize, _len: usize) { todo!() }
}

impl InputStream {
    pub fn read(&self, _buffer: &mut [u8]) -> i32 { todo!() }
    pub fn read_off(&self, _buffer: &mut [u8], _off: usize, _len: usize) -> i32 { todo!() }
}

impl Charset {
    pub fn default_charset() -> Self { todo!() }
    pub fn for_name(_name: &str) -> Self { todo!() }
    pub fn name(&self) -> String { todo!() }
}

impl InputStreamReader {
    pub fn new(_input: &InputStream, _charset_name: String) -> Self { todo!() }
}

impl OutputStreamWriter {
    pub fn new(_output: &mut OutputStream, _charset_name: String) -> Self { todo!() }
    pub fn flush(&mut self) { todo!() }
}

impl Reader {
    pub fn read(&self, _buffer: &mut CharBuffer) -> i32 { todo!() }
    pub fn read_char(&self, _buffer: &mut [char], _off: usize, _len: usize) -> i32 { todo!() }
}

impl Writer {
    pub fn write(&mut self, _buffer: &[char], _off: usize, _len: usize) { todo!() }
}

impl Appendable {
    pub fn append(&mut self, _buffer: &CharBuffer, _off: usize, _len: usize) { todo!() }
}

impl CharBuffer {
    pub fn allocate(_size: usize) -> Self { todo!() }
    pub fn flip(&mut self) { todo!() }
}

impl ReadableByteChannel {
    pub fn read(&mut self, _buffer: &mut ByteBuffer) -> i32 { todo!() }
}

impl ByteBuffer {
    pub fn allocate(_size: usize) -> Self { todo!() }
    pub fn position(&mut self, _pos: usize) { todo!() }
    pub fn limit(&mut self, _limit: usize) { todo!() }
}

impl Closeable {
    pub fn close(&mut self) -> Result<(), io::Error> { todo!() }
}

impl URLConnection {
    pub fn is_http_url_connection(&self) -> bool { todo!() }
    pub fn disconnect(&self) { todo!() }
}
